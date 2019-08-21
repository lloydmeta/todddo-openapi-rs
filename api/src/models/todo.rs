use domain::todo as domain_models;
use paperclip::actix::api_v2_schema;
use serde_derive::{Deserialize, Serialize};

#[api_v2_schema(empty)]
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Copy, Clone)]
pub struct TodoId(pub u64);

#[api_v2_schema]
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoData {
    pub task: String,
}

#[api_v2_schema]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Todo {
    pub id: TodoId,
    pub task: String,
}

impl From<&TodoId> for domain_models::TodoId {
    fn from(v: &TodoId) -> Self {
        domain_models::TodoId(v.0)
    }
}

impl From<&TodoData> for domain_models::TodoData {
    fn from(v: &TodoData) -> Self {
        domain_models::TodoData {
            task: v.task.clone(),
        }
    }
}

impl From<&Todo> for domain_models::Todo {
    fn from(v: &Todo) -> Self {
        domain_models::Todo {
            id: (&v.id).into(),
            task: v.task.clone(),
        }
    }
}

impl From<domain_models::TodoId> for TodoId {
    fn from(v: domain_models::TodoId) -> Self {
        TodoId(v.0)
    }
}

impl From<domain_models::TodoData> for TodoData {
    fn from(v: domain_models::TodoData) -> Self {
        TodoData { task: v.task }
    }
}

impl From<domain_models::Todo> for Todo {
    fn from(v: domain_models::Todo) -> Self {
        Todo {
            id: v.id.into(),
            task: v.task,
        }
    }
}
