use crate::todo::*;

use async_trait::async_trait;

#[async_trait]
pub trait TodoService {
    async fn create(&self, todo_data: &TodoData) -> Result<Todo, TodoServiceDataErr>;
    async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoServiceLookupErr>;
    async fn list(&self) -> Vec<Todo>;
    async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoServiceLookupErr>;
    async fn update(&self, todo: &Todo) -> Result<(), TodoServiceUpdateErr>;
}

pub struct TodoServiceImpl<A: TodoRepo + Sync> {
    todo_repo: A,
}

pub fn new<A: TodoRepo + Sync>(repo: A) -> TodoServiceImpl<A> {
    TodoServiceImpl { todo_repo: repo }
}

impl<A: TodoRepo + Sync> TodoServiceImpl<A> {
    fn validate_task(task: &str) -> Result<(), TodoServiceDataErr> {
        if task.is_empty() {
            Err(TodoServiceDataErr::InvalidData {
                task: task.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<A: TodoRepo + Sync> TodoService for TodoServiceImpl<A> {
    async fn create(&self, todo_data: &TodoData) -> Result<Todo, TodoServiceDataErr> {
        Self::validate_task(&todo_data.task)?;
        Ok(self.todo_repo.create(todo_data).await)
    }

    async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoServiceLookupErr> {
        Ok(self.todo_repo.get(todo_id).await?)
    }

    async fn list(&self) -> Vec<Todo> {
        self.todo_repo.list().await
    }

    async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoServiceLookupErr> {
        Ok(self.todo_repo.delete(todo_id).await?)
    }

    async fn update(&self, todo: &Todo) -> Result<(), TodoServiceUpdateErr> {
        Self::validate_task(&todo.task)?;
        Ok(self.todo_repo.update(todo).await?)
    }
}

pub enum TodoServiceUpdateErr {
    LookupErr(TodoServiceLookupErr),
    DataErr(TodoServiceDataErr),
}

pub enum TodoServiceLookupErr {
    NotFound(TodoId),
}

pub enum TodoServiceDataErr {
    InvalidData { task: String },
}

impl From<TodoRepoErr> for TodoServiceLookupErr {
    fn from(repo_err: TodoRepoErr) -> Self {
        match repo_err {
            TodoRepoErr::NotFound(id) => TodoServiceLookupErr::NotFound(id),
        }
    }
}

impl From<TodoServiceDataErr> for TodoServiceUpdateErr {
    fn from(err: TodoServiceDataErr) -> Self {
        TodoServiceUpdateErr::DataErr(err)
    }
}

impl From<TodoRepoErr> for TodoServiceUpdateErr {
    fn from(repo_err: TodoRepoErr) -> Self {
        TodoServiceUpdateErr::LookupErr(repo_err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::*;

    #[test]
    fn test_create_ok() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let f_created = async {
            let todo_data = TodoData {
                task: "Make the bed".to_string(),
            };
            service.create(&todo_data).await
        };
        match block_on(f_created) {
            Ok(saved) => {
                assert_eq!("Make the bed".to_string(), saved.task);
                assert_eq!(1, *mock_repo.create_called.lock().unwrap());
            }
            Err(_) => panic!("Creation failed"),
        }
    }

    #[test]
    fn test_create_invalid() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let f_created = async {
            let todo_data = TodoData {
                task: "".to_string(),
            };
            service.create(&todo_data).await
        };
        match block_on(f_created) {
            Err(TodoServiceDataErr::InvalidData { .. }) => {
                assert_eq!(0, *mock_repo.create_called.lock().unwrap());
            }
            Ok(_) => panic!("invalid data was saved"),
        }
    }

    #[test]
    fn test_get_ok() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        match block_on(service.get(&TodoId(1))) {
            Ok(_) => {
                assert_eq!(1, *mock_repo.get_called.lock().unwrap());
            }
            Err(_) => panic!("not found"),
        }
    }

    #[test]
    fn test_get_not_found() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        match block_on(service.get(&NOT_FOUND_TODO_ID)) {
            Err(TodoServiceLookupErr::NotFound { .. }) => {
                assert_eq!(1, *mock_repo.get_called.lock().unwrap());
            }
            Ok(_) => panic!("not found"),
        }
    }

    #[test]
    fn test_list() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let _ = block_on(service.list());
        assert_eq!(1, *mock_repo.list_called.lock().unwrap());
    }

    #[test]
    fn test_delete_ok() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        match block_on(service.delete(&TodoId(1))) {
            Ok(_) => {
                assert_eq!(1, *mock_repo.delete_called.lock().unwrap());
            }
            Err(_) => panic!("not found"),
        }
    }

    #[test]
    fn test_delete_not_found() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        match block_on(service.delete(&NOT_FOUND_TODO_ID)) {
            Err(TodoServiceLookupErr::NotFound { .. }) => {
                assert_eq!(1, *mock_repo.delete_called.lock().unwrap());
            }
            Ok(_) => panic!("not found"),
        }
    }

    #[test]
    fn test_update_ok() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let update_data = Todo {
            id: TodoId(1),
            task: "hello".to_string(),
        };
        match block_on(service.update(&update_data)) {
            Ok(_) => {
                assert_eq!(1, *mock_repo.update_called.lock().unwrap());
            }
            Err(_) => panic!("eh wut"),
        }
    }

    #[test]
    fn test_update_not_found() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let update_data = Todo {
            id: NOT_FOUND_TODO_ID,
            task: "hello".to_string(),
        };
        match block_on(service.update(&update_data)) {
            Err(TodoServiceUpdateErr::LookupErr(_)) => {
                assert_eq!(1, *mock_repo.update_called.lock().unwrap());
            }
            _ => panic!("Unexpected."),
        }
    }

    #[test]
    fn test_update_invalid_data() {
        let mock_repo = MockTodoRepo::new();
        let service = new(mock_repo.clone());
        let update_data = Todo {
            id: TodoId(1),
            task: "".to_string(),
        };
        match block_on(service.update(&update_data)) {
            Err(TodoServiceUpdateErr::DataErr(_)) => {
                assert_eq!(0, *mock_repo.update_called.lock().unwrap());
            }
            _ => panic!("Unexpected."),
        }
    }

    #[derive(Clone)]
    struct MockTodoRepo {
        create_called: Arc<Mutex<usize>>,
        update_called: Arc<Mutex<usize>>,
        get_called: Arc<Mutex<usize>>,
        list_called: Arc<Mutex<usize>>,
        delete_called: Arc<Mutex<usize>>,
    }

    impl MockTodoRepo {
        fn new() -> MockTodoRepo {
            MockTodoRepo {
                create_called: Arc::new(Mutex::new(0)),
                update_called: Arc::new(Mutex::new(0)),
                get_called: Arc::new(Mutex::new(0)),
                list_called: Arc::new(Mutex::new(0)),
                delete_called: Arc::new(Mutex::new(0)),
            }
        }
    }

    static NOT_FOUND_TODO_ID: TodoId = TodoId(999);
    static RETRIEVED_TODO_TASK: &str = "say hello";

    #[async_trait]
    impl TodoRepo for MockTodoRepo {
        async fn create(&self, todo_data: &TodoData) -> Todo {
            let mut mutex = self.create_called.lock().unwrap();
            *mutex += 1;
            let saved = Todo {
                id: TodoId(1),
                task: todo_data.task.clone(),
            };
            saved
        }

        async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoRepoErr> {
            let mut mutex = self.get_called.lock().unwrap();
            *mutex += 1;
            if *todo_id == NOT_FOUND_TODO_ID {
                Err(TodoRepoErr::NotFound(*todo_id))
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

        async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoRepoErr> {
            let mut mutex = self.delete_called.lock().unwrap();
            *mutex += 1;
            if todo_id == &NOT_FOUND_TODO_ID {
                Err(TodoRepoErr::NotFound(*todo_id))
            } else {
                Ok(())
            }
        }

        async fn update(&self, todo: &Todo) -> Result<(), TodoRepoErr> {
            let mut mutex = self.update_called.lock().unwrap();
            *mutex += 1;
            if &todo.id == &NOT_FOUND_TODO_ID {
                Err(TodoRepoErr::NotFound(todo.id))
            } else {
                Ok(())
            }
        }
    }
}
