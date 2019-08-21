use async_trait::async_trait;

#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Copy, Clone, Hash)]
pub struct TodoId(pub u64);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TodoData {
    pub task: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Todo {
    pub id: TodoId,
    pub task: String,
}

// The algebra for a [[Todo]] repository, dealing w/ persistence
#[async_trait]
pub trait TodoRepo {
    async fn create(&self, todo_data: &TodoData) -> Todo;
    async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoRepoErr>;
    async fn list(&self) -> Vec<Todo>;
    async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoRepoErr>;
    async fn update(&self, todo: &Todo) -> Result<(), TodoRepoErr>;
}

pub enum TodoRepoErr {
    NotFound(TodoId),
}
