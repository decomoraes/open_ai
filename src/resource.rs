use std::sync::{Arc, Mutex};
use crate::core::APIClient;

// pub type APIResource = Rc<RefCell<APIClient>>;
pub type APIResource = Arc<Mutex<APIClient>>;