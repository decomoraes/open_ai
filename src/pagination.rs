// use std::pin::Pin;
// use std::task::{Context, Poll};
// use futures::future::BoxFuture;
// use futures::Stream;
// use reqwest::Method;
// use serde::Serialize;
// use crate::core::{AbstractPage, APIClient, APIResponseProps, FinalRequestOptions, PageInfo};
//
// #[derive(Debug)]
// pub struct PageResponse<Item> {
//   data: Vec<Item>,
//   object: String,
// }
//
// /// Note: no pagination actually occurs yet, this is for forwards-compatibility.
// #[derive(Debug)]
// pub struct Page<Item> where Item: Clone + Serialize { // extends AbstractPage<Item> implements PageResponse<Item> {
//   data: Vec<Item>,
//   object: String,
//
//   body: PageResponse<Item>,
//   client: APIClient,
//   options: FinalRequestOptions,
//   response: reqwest::Response,
//   // body: unknown,
// }
//
// impl<Item: Clone> Stream<Item=Item> for Page<Item> {
//   type Item = Item;
//
//   fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//     todo!()
//   }
// }
//
// impl<Item: Clone + Serialize> AbstractPage<Item> for Page<Item> {
//   fn new(client: APIClient, response: APIResponseProps) -> Self {
//     todo!()
//   }
//
//   fn has_next_page(&self) -> bool {
//     let items = self.get_paginated_items();
//     if items.len() == 0 {
//       return false;
//     }
//     self.next_page_info().is_some()
//   }
//
//   fn get_next_page(&self) -> BoxFuture<'static, Self>
//   where
//       Self: Sized,
//   {
//     let client = self.client.clone();
//     let next_info = self.next_page_info().unwrap(); // Handle the case when next_page_info is None
//     let options = self.options.clone(); // Clone the options
//
//     Box::pin(async move {
//       let new_options = if let PageInfo::Params(params) = next_info {
//         let mut query = options.request_options.query.unwrap_or_default();
//         if let Some(params) = params {
//           for (key, value) in params {
//             query[key] = value;
//           }
//         }
//         let mut new_options = options.clone();
//         new_options.request_options.query = Some(query);
//         new_options
//       } else {
//         options.clone()
//       };
//
//       let response_props = client.request_api_list::<Item, Self>(&options.path, new_options).await;
//
//       // let opts: FinalRequestOptions::new(Method::GET, options.request_options);
//       Page::new(client, response_props, PageResponse { data: vec![], object: "".to_string() })
//     })
//   }
//
//   fn get_paginated_items(&mut self) -> Vec<Item> {
//     todo!()
//   }
// }
//
// impl<Item: Clone> Page<Item> {
//   fn new(
//     client: APIClient,
//     response: reqwest::Response,
//     body: PageResponse<Item>,
//     options: FinalRequestOptions
//   ) -> Self {
//     let page = Page {
//       data: body.data.clone(),
//       object: body.object.clone(),
//       body,
//       client,
//       options,
//       response,
//     };
//     page
//   }
//
//   pub fn get_paginated_items(&self) -> Vec<Item> {
//     self.data.clone()
//   }
//
//   /// This page represents a response that isn't actually paginated at the API level
//   /// so there will never be any next page params.
//   #[deprecated(note = "Please use `nextPageInfo()` instead")]
//   pub fn next_page_params(&self) -> Option<()> {
//     return None;
//   }
//
//   pub fn next_page_info(&self) -> Option<PageInfo> {
//     return None;
//   }
// }
//
// pub struct CursorPageResponse<Item> {
//   data: Vec<Item>,
// }
//
// pub struct CursorPageParams {
//   after: Option<String>,
//
//   limit: Option<u32>,
// }

/////////////////////////////////////////////////////////////////

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use futures::Stream;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::core::{APIClient, FinalRequestOptions, RequestOptions, PageInfo};
use crate::resource::APIResource;
use crate::resources::beta::assistants::Assistant;
use crate::resources::beta::threads::{Run, RunListParams};

pub trait Page<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de>>: Sized {
    fn new(client: Rc<RefCell<APIClient>>, /*response: reqwest::Response,*/ body: CursorPageResponse<Item>, options: FinalRequestOptions<Req>) -> Self;
    fn next_page_info(&self) -> Option<PageInfo>;
    fn get_paginated_items(&self) -> Vec<Item>;
    fn has_next_page(&self) -> bool;
    async fn get_next_page(&self) -> Result<Self, Box<dyn Error>>;
    async fn iter_pages(&self) -> &Self;
    // async fn iter_pages(&mut self) -> impl Iterator<Item = Result<Self, Box<dyn Error>>> + '_
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CursorPageResponse<Item> {
    object: CursorPageResponseObject,
    data: Vec<Item>,
    first_id: String,
    last_id: String,
    has_more: bool
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CursorPageResponseObject {
    #[default]
    List,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CursorPageParams {
    pub after: String,
    pub limit: u32,
}

#[derive(Debug)]
pub struct CursorPage<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de>> { // <Item extends { id: string }>
    pub data: Vec<Item>,
    pub client: Rc<RefCell<APIClient>>,
    pub options: FinalRequestOptions<Req>,
    // pub response: Response,
    pub body: CursorPageResponse<Item>,
}

// extends AbstractPage<Item>
// implements CursorPageResponse<Item>

impl<Req: Default + Clone + Serialize, Item: for<'de> Deserialize<'de> + Clone> Page<Req, Item> for CursorPage<Req, Item> {
    fn new(
        client: Rc<RefCell<APIClient>>,
        // response: reqwest::Response,
        body: CursorPageResponse<Item>,
        options: FinalRequestOptions<Req>,
    ) -> Self {
        // super(client, response, body, options);

        // this.data = body.data || [];
        CursorPage {
            client: client,
            // response: response,
            data: body.data.clone(),
            body: body,
            options: options,
        }
    }

    fn next_page_info(&self) -> Option<PageInfo> {
        let data = self.get_paginated_items();
        if data.len() == 0 {
            return None;
        }

        // let id = data[data.len() - 1]?.id;
        // if !id {
        //     return None;
        // }

        // let mut hash_map: HashMap<String, Value> = HashMap::new();
        // hash_map.insert("after".to_string(), Value::String(id));
        // Some(PageInfo::Params(Some(hash_map)))
        None
    }

    fn get_paginated_items(&self) -> Vec<Item> {
        // self.data.clone()
        vec![]
    }

    fn has_next_page(&self) -> bool {
        let items = self.get_paginated_items();
        if items.len() == 0 {
            return false;
        }
        return self.next_page_info().is_some();
    }

    async fn get_next_page(&self) -> Result<Self, Box<dyn Error>> {
        let next_info = self.next_page_info();
        if next_info.is_none() {
            // throw new OpenAIError(
            //     'No next page expected; please check `.hasNextPage()` before calling `.getNextPage()`.',
            // );
            // return self
        }
        let mut next_info = next_info.unwrap();
        let mut next_options = self.options.clone();
        match next_info {
            PageInfo::Params(params) => {
                // let mut query = next_options.query.unwrap_or_default();
                // if let Some(params) = params {
                //     for (key, value) in params {
                //         // query.insert(key, value);
                //     }
                // }
                // next_options.query = Some(query);
                return Err("Not implemented".into());
            }
            PageInfo::Url(url) => {
                next_options.query = None;
                next_options.path = url.to_string();
            }
        }

        let page_constructor = |
            client: APIResource,
            body: CursorPageResponse<Item>,
            options: FinalRequestOptions<Req>,
        | {
            CursorPage::new(client, body, options)
        };

        let result = self.client.as_ref().borrow().request_api_list(page_constructor, next_options).await.unwrap();
        
        Ok(result)
    }

    async fn iter_pages(&self) -> &Self {
        self
    }
}
