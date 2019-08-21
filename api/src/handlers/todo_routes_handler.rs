use crate::controllers::todo_controller::*;
use crate::models::common::Message;
use crate::models::todo::{Todo, TodoData, TodoId};
use actix_web::*;
use futures::future::{FutureExt, TryFutureExt};
use futures_01::Future as Future01;
use paperclip::actix::{api_v2_operation, api_v2_schema};
use std::ops::Deref;

#[api_v2_operation]
pub fn list<A: TodoController + Send + Sync + 'static>(
    web: web::Data<A>,
) -> impl Future01<Item = web::Json<Vec<Todo>>, Error = Error> {
    let f_resp = async move {
        let controller = web.get_ref();
        let listed = controller.list().await;
        Ok(web::Json(listed))
    };
    f_resp.boxed().compat()
}

#[api_v2_operation]
pub fn create<A: TodoController + Send + Sync + 'static>(
    web: web::Data<A>,
    json: web::Json<TodoData>,
) -> impl Future01<Item = web::Json<Todo>, Error = TodoRoutesError> {
    let f_resp = async move {
        let controller = web.get_ref();
        let todo = controller.create(json.deref()).await?;
        Ok(web::Json(todo))
    };
    f_resp.boxed().compat()
}

#[api_v2_operation]
pub fn get<A: TodoController + Send + Sync + 'static>(
    web: web::Data<A>,
    id: web::Path<TodoId>,
) -> impl Future01<Item = web::Json<Todo>, Error = TodoRoutesError> {
    let f_resp = async move {
        let controller = web.get_ref();
        let get_result = controller.get(id.deref().into()).await?;
        Ok(web::Json(get_result))
    };
    f_resp.boxed().compat()
}

#[api_v2_operation]
pub fn delete<A: TodoController + Send + Sync + 'static>(
    web: web::Data<A>,
    id: web::Path<TodoId>,
) -> impl Future01<Item = web::Json<Message>, Error = TodoRoutesError> {
    let f_resp = async move {
        let controller = web.get_ref();
        let _ = controller.delete(id.deref()).await?;
        Ok(web::Json(Message {
            message: format!("Successfully deleted: [{:?}]", id),
        }))
    };
    f_resp.boxed().compat()
}

#[api_v2_operation]
pub fn update<A: TodoController + Send + Sync + 'static>(
    web: web::Data<A>,
    id: web::Path<TodoId>,
    json: web::Json<TodoData>,
) -> impl Future01<Item = web::Json<Message>, Error = TodoRoutesError> {
    let f_resp = async move {
        let controller = web.get_ref();
        let todo = Todo {
            id: *id.deref(),
            task: json.into_inner().task,
        };
        let _ = controller.update(&todo).await?;
        Ok(web::Json(Message {
            message: format!("Successfully updated: [{:?}]", id),
        }))
    };
    f_resp.boxed().compat()
}

use failure::Fail;

#[api_v2_schema]
#[derive(Fail, Debug)]
pub enum TodoRoutesError {
    #[fail(display = "Bad task data")]
    BadTask { task: String },
    #[fail(display = "No such task")]
    NoSuchTask { id: TodoId },
}

use TodoRoutesError::*;

impl error::ResponseError for TodoRoutesError {
    fn error_response(&self) -> HttpResponse {
        match self {
            BadTask { task } => HttpResponse::BadRequest().json(&Message {
                message: format!("Invalid task: [{}]", task),
            }),
            NoSuchTask { id } => HttpResponse::NotFound().json(&Message {
                message: format!("No such todo: [{:?}]", id),
            }),
        }
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}

impl From<TodoControllerDataErr> for TodoRoutesError {
    fn from(e: TodoControllerDataErr) -> Self {
        match e {
            TodoControllerDataErr::InvalidData { task } => TodoRoutesError::BadTask { task },
        }
    }
}

impl From<TodoControllerLookupErr> for TodoRoutesError {
    fn from(e: TodoControllerLookupErr) -> Self {
        match e {
            TodoControllerLookupErr::NotFound(id) => TodoRoutesError::NoSuchTask { id: id.into() },
        }
    }
}

impl From<TodoControllerUpdateErr> for TodoRoutesError {
    fn from(e: TodoControllerUpdateErr) -> Self {
        match e {
            TodoControllerUpdateErr::LookupErr(e) => e.into(),
            TodoControllerUpdateErr::DataErr(e) => e.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use async_trait::async_trait;
    use std::sync::*;

    static RETURNED_TASK: &str = "say hello";

    fn expected_task() -> Todo {
        Todo {
            id: TodoId(1),
            task: RETURNED_TASK.to_string(),
        }
    }

    #[test]
    fn test_create() {
        let mock_controller = MockTodoController::new();
        let todo_data = TodoData {
            task: "say goodbye".to_string(),
        };
        let todo_json = web::Json(todo_data);
        let req = test::TestRequest::default()
            .data(mock_controller.clone())
            .to_http_request();
        let app_data = req.get_app_data().unwrap();
        let resp = test::block_on(create::<MockTodoController>(app_data, todo_json))
            .unwrap()
            .0;
        assert_eq!("say goodbye", &resp.task);
        let times_called = *mock_controller.create_called.lock().unwrap();
        assert_eq!(1, times_called);
    }

    #[test]
    fn test_get() {
        let mock_controller = MockTodoController::new();
        let req = test::TestRequest::default()
            .data(mock_controller.clone())
            .to_http_request();
        let app_data = req.get_app_data().unwrap();
        let id = TodoId(123);
        let resp = test::block_on(get::<MockTodoController>(app_data, id.into()))
            .unwrap()
            .0;
        assert_eq!(id, resp.id);
        let times_called = *mock_controller.get_called.lock().unwrap();
        assert_eq!(1, times_called);
    }

    #[test]
    fn test_list() {
        let mock_controller = MockTodoController::new();
        let req = test::TestRequest::default()
            .data(mock_controller.clone())
            .to_http_request();
        let app_data = req.get_app_data().unwrap();
        let resp = test::block_on(list::<MockTodoController>(app_data))
            .unwrap()
            .0;
        assert_eq!(vec![expected_task()], resp);
        let times_called = *mock_controller.list_called.lock().unwrap();
        assert_eq!(1, times_called);
    }

    #[test]
    fn test_delete() {
        let mock_controller = MockTodoController::new();
        let req = test::TestRequest::default()
            .data(mock_controller.clone())
            .to_http_request();
        let app_data = req.get_app_data().unwrap();
        let id = TodoId(123);
        let _ = test::block_on(delete::<MockTodoController>(app_data, id.into()))
            .unwrap()
            .0;
        let times_called = *mock_controller.delete_called.lock().unwrap();
        assert_eq!(1, times_called);
    }

    #[test]
    fn test_update() {
        let mock_controller = MockTodoController::new();
        let todo_data = TodoData {
            task: "say goodbye".to_string(),
        };
        let todo_json = web::Json(todo_data);
        let req = test::TestRequest::default()
            .data(mock_controller.clone())
            .to_http_request();
        let app_data = req.get_app_data().unwrap();
        let id = TodoId(123);
        let _ = test::block_on(update::<MockTodoController>(app_data, id.into(), todo_json))
            .unwrap()
            .0;
        let times_called = *mock_controller.update_called.lock().unwrap();
        assert_eq!(1, times_called);
    }

    #[derive(Clone)]
    struct MockTodoController {
        create_called: Arc<Mutex<usize>>,
        update_called: Arc<Mutex<usize>>,
        get_called: Arc<Mutex<usize>>,
        list_called: Arc<Mutex<usize>>,
        delete_called: Arc<Mutex<usize>>,
    }

    impl MockTodoController {
        fn new() -> MockTodoController {
            MockTodoController {
                create_called: Arc::new(Mutex::new(0)),
                update_called: Arc::new(Mutex::new(0)),
                get_called: Arc::new(Mutex::new(0)),
                list_called: Arc::new(Mutex::new(0)),
                delete_called: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl TodoController for MockTodoController {
        async fn create(&self, todo_data: &TodoData) -> Result<Todo, TodoControllerDataErr> {
            let mut mutex = self.create_called.lock().unwrap();
            *mutex += 1;
            Ok(Todo {
                id: TodoId(123),
                task: todo_data.task.clone(),
            })
        }

        async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoControllerLookupErr> {
            let mut mutex = self.get_called.lock().unwrap();
            *mutex += 1;
            Ok(Todo {
                id: *todo_id,
                task: RETURNED_TASK.to_string(),
            })
        }

        async fn list(&self) -> Vec<Todo> {
            let mut mutex = self.list_called.lock().unwrap();
            *mutex += 1;
            vec![expected_task()]
        }

        async fn update(&self, _: &Todo) -> Result<(), TodoControllerUpdateErr> {
            let mut mutex = self.update_called.lock().unwrap();
            *mutex += 1;
            Ok(())
        }

        async fn delete(&self, _: &TodoId) -> Result<(), TodoControllerLookupErr> {
            let mut mutex = self.delete_called.lock().unwrap();
            *mutex += 1;
            Ok(())
        }
    }
}
