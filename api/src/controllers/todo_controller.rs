use crate::models::todo as api_models;
use async_trait::async_trait;
use domain::services::todo_service::{
    TodoService, TodoServiceDataErr, TodoServiceLookupErr, TodoServiceUpdateErr,
};

#[async_trait]
pub trait TodoController {
    async fn create(
        &self,
        todo_data: &api_models::TodoData,
    ) -> Result<api_models::Todo, TodoControllerDataErr>;
    async fn get(
        &self,
        todo_id: &api_models::TodoId,
    ) -> Result<api_models::Todo, TodoControllerLookupErr>;
    async fn list(&self) -> Vec<api_models::Todo>;
    async fn update(&self, todo: &api_models::Todo) -> Result<(), TodoControllerUpdateErr>;
    async fn delete(&self, todo_id: &api_models::TodoId) -> Result<(), TodoControllerLookupErr>;
}

#[derive(Clone)]
pub struct TodoControllerImpl<A: TodoService + Sync> {
    todo_service: A,
}

pub fn new<A: TodoService + Sync>(todo_service: A) -> TodoControllerImpl<A> {
    TodoControllerImpl { todo_service }
}

#[async_trait]
impl<A: TodoService + Sync> TodoController for TodoControllerImpl<A> {
    async fn create(
        &self,
        todo_data: &api_models::TodoData,
    ) -> Result<api_models::Todo, TodoControllerDataErr> {
        let as_domain_data = todo_data.into();
        let domain_todo = self.todo_service.create(&as_domain_data).await?;
        Ok(domain_todo.into())
    }

    async fn get(
        &self,
        todo_id: &api_models::TodoId,
    ) -> Result<api_models::Todo, TodoControllerLookupErr> {
        let domain_id = todo_id.into();
        let domain_todo = self.todo_service.get(&domain_id).await?;
        Ok(domain_todo.into())
    }

    async fn list(&self) -> Vec<api_models::Todo> {
        let domain_todos = self.todo_service.list().await;
        domain_todos.into_iter().map(|v| v.into()).collect()
    }

    async fn update(&self, todo: &api_models::Todo) -> Result<(), TodoControllerUpdateErr> {
        let as_domain_todo = todo.into();
        Ok(self.todo_service.update(&as_domain_todo).await?)
    }

    async fn delete(&self, todo_id: &api_models::TodoId) -> Result<(), TodoControllerLookupErr> {
        let domain_id = todo_id.into();
        Ok(self.todo_service.delete(&domain_id).await?)
    }
}

pub enum TodoControllerUpdateErr {
    LookupErr(TodoControllerLookupErr),
    DataErr(TodoControllerDataErr),
}

pub enum TodoControllerLookupErr {
    NotFound(api_models::TodoId),
}

impl From<TodoServiceLookupErr> for TodoControllerLookupErr {
    fn from(e: TodoServiceLookupErr) -> Self {
        match e {
            TodoServiceLookupErr::NotFound(id) => TodoControllerLookupErr::NotFound(id.into()),
        }
    }
}

pub enum TodoControllerDataErr {
    InvalidData { task: String },
}

impl From<TodoServiceDataErr> for TodoControllerDataErr {
    fn from(e: TodoServiceDataErr) -> Self {
        match e {
            TodoServiceDataErr::InvalidData { task } => TodoControllerDataErr::InvalidData { task },
        }
    }
}

impl From<TodoServiceUpdateErr> for TodoControllerUpdateErr {
    fn from(err: TodoServiceUpdateErr) -> Self {
        match err {
            TodoServiceUpdateErr::DataErr(inner) => TodoControllerUpdateErr::DataErr(inner.into()),
            TodoServiceUpdateErr::LookupErr(inner) => {
                TodoControllerUpdateErr::LookupErr(inner.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::todo::{Todo, TodoData, TodoId};
    use futures::executor::block_on;
    use std::sync::*;

    static NOT_FOUND_TODO_ID: api_models::TodoId = api_models::TodoId(999);
    static RETRIEVED_TODO_TASK: &str = "say hello";
    static INVALID_TASK: &str = "reflection";

    #[test]
    fn test_create_ok() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_created = async {
            let todo_data = api_models::TodoData {
                task: "say hello".to_string(),
            };
            controller.create(&todo_data).await
        };
        match block_on(f_created) {
            Ok(saved) => {
                assert_eq!("say hello", &saved.task);
                assert_eq!(1, *mock_service.create_called.lock().unwrap());
            }
            _ => panic!("creation failed"),
        }
    }

    #[test]
    fn test_create_invalid_data() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_created = async {
            let todo_data = api_models::TodoData {
                task: INVALID_TASK.to_string(),
            };
            controller.create(&todo_data).await
        };
        match block_on(f_created) {
            Err(_) => {
                assert_eq!(0, *mock_service.create_called.lock().unwrap());
            }
            _ => panic!("creation failed"),
        }
    }

    #[test]
    fn test_get_ok() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_retrieved = async { controller.get(&api_models::TodoId(123)).await };
        match block_on(f_retrieved) {
            Ok(_) => {
                assert_eq!(1, *mock_service.get_called.lock().unwrap());
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_get_not_found() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_retrieved = async { controller.get(&NOT_FOUND_TODO_ID).await };
        match block_on(f_retrieved) {
            Err(_) => {
                assert_eq!(1, *mock_service.get_called.lock().unwrap());
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_list() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_listed = async { controller.list().await };
        assert_eq!(
            vec![api_models::Todo {
                id: api_models::TodoId(1),
                task: RETRIEVED_TODO_TASK.to_string(),
            }],
            block_on(f_listed)
        );
        assert_eq!(1, *mock_service.list_called.lock().unwrap());
    }

    #[test]
    fn test_delete_ok() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_deleted = async { controller.delete(&api_models::TodoId(123)).await };
        match block_on(f_deleted) {
            Ok(_) => {
                assert_eq!(1, *mock_service.delete_called.lock().unwrap());
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_delete_not_found() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_deleted = async { controller.delete(&NOT_FOUND_TODO_ID).await };
        match block_on(f_deleted) {
            Err(_) => {
                assert_eq!(1, *mock_service.delete_called.lock().unwrap());
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_update_ok() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_updated = async {
            let todo = api_models::Todo {
                id: api_models::TodoId(1),
                task: "hello world".to_string(),
            };
            controller.update(&todo).await
        };
        match block_on(f_updated) {
            Ok(_) => {
                assert_eq!(1, *mock_service.update_called.lock().unwrap());
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_update_not_found() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_updated = async {
            let todo = api_models::Todo {
                id: NOT_FOUND_TODO_ID.into(),
                task: "hello world".to_string(),
            };
            controller.update(&todo).await
        };
        match block_on(f_updated) {
            Err(TodoControllerUpdateErr::LookupErr(_)) => {
                assert_eq!(1, *mock_service.update_called.lock().unwrap())
            }
            _ => panic!("lookup failed"),
        }
    }

    #[test]
    fn test_update_invalid_data() {
        let mock_service = MockTodoService::new();
        let controller = new(mock_service.clone());
        let f_updated = async {
            let todo = api_models::Todo {
                id: api_models::TodoId(1),
                task: INVALID_TASK.to_string(),
            };
            controller.update(&todo).await
        };
        match block_on(f_updated) {
            Err(TodoControllerUpdateErr::DataErr(_)) => {
                assert_eq!(1, *mock_service.update_called.lock().unwrap())
            }
            _ => panic!("lookup failed"),
        }
    }

    #[derive(Clone)]
    struct MockTodoService {
        create_called: Arc<Mutex<usize>>,
        update_called: Arc<Mutex<usize>>,
        get_called: Arc<Mutex<usize>>,
        list_called: Arc<Mutex<usize>>,
        delete_called: Arc<Mutex<usize>>,
    }

    impl MockTodoService {
        fn new() -> MockTodoService {
            MockTodoService {
                create_called: Arc::new(Mutex::new(0)),
                update_called: Arc::new(Mutex::new(0)),
                get_called: Arc::new(Mutex::new(0)),
                list_called: Arc::new(Mutex::new(0)),
                delete_called: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl TodoService for MockTodoService {
        async fn create(&self, todo_data: &TodoData) -> Result<Todo, TodoServiceDataErr> {
            if todo_data.task == INVALID_TASK {
                Err(TodoServiceDataErr::InvalidData {
                    task: todo_data.task.clone(),
                })
            } else {
                let mut mutex = self.create_called.lock().unwrap();
                *mutex += 1;
                let saved = Todo {
                    id: TodoId(1),
                    task: todo_data.task.clone(),
                };
                Ok(saved)
            }
        }

        async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoServiceLookupErr> {
            let mut mutex = self.get_called.lock().unwrap();
            *mutex += 1;
            if todo_id.0 == NOT_FOUND_TODO_ID.0 {
                Err(TodoServiceLookupErr::NotFound(*todo_id))
            } else {
                Ok(Todo {
                    id: *todo_id,
                    task: RETRIEVED_TODO_TASK.to_string(),
                })
            }
        }

        async fn list(&self) -> Vec<Todo> {
            let mut mutex = self.list_called.lock().unwrap();
            *mutex += 1;
            vec![Todo {
                id: TodoId(1),
                task: RETRIEVED_TODO_TASK.to_string(),
            }]
        }

        async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoServiceLookupErr> {
            let mut mutex = self.delete_called.lock().unwrap();
            *mutex += 1;
            if todo_id.0 == NOT_FOUND_TODO_ID.0 {
                Err(TodoServiceLookupErr::NotFound(*todo_id))
            } else {
                Ok(())
            }
        }

        async fn update(&self, todo: &Todo) -> Result<(), TodoServiceUpdateErr> {
            let mut mutex = self.update_called.lock().unwrap();
            *mutex += 1;
            if todo.task == INVALID_TASK {
                Err(TodoServiceUpdateErr::DataErr(
                    TodoServiceDataErr::InvalidData {
                        task: todo.task.clone(),
                    },
                ))
            } else if todo.id.0 == NOT_FOUND_TODO_ID.0 {
                Err(TodoServiceUpdateErr::LookupErr(
                    TodoServiceLookupErr::NotFound(todo.id),
                ))
            } else {
                Ok(())
            }
        }
    }
}
