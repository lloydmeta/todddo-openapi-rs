use domain::todo::*;
use futures_locks::{Mutex, MutexGuard};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use async_trait::async_trait;
use futures::compat::Future01CompatExt;

#[derive(Clone)]
pub struct InMemTodoRepo {
    data: Mutex<Data>,
}

pub fn new() -> InMemTodoRepo {
    InMemTodoRepo {
        data: Mutex::new(Data {
            last_id: LastId(0),
            storage: HashMap::new(),
        }),
    }
}

impl InMemTodoRepo {
    async fn unlock(&self) -> MutexGuard<Data> {
        let guard = self.data.lock().compat().await;
        guard.expect("Removing compat layer Result")
    }
}

#[async_trait]
impl TodoRepo for InMemTodoRepo {
    async fn create(&self, todo_data: &TodoData) -> Todo {
        let mut data = self.unlock().await;
        let next_id = data.last_id.0 + 1;
        let id = TodoId(next_id);
        data.last_id = LastId(next_id);
        let persistable_todo = PersistedTodo {
            task: todo_data.task.clone(),
        };
        data.storage.insert(id, persistable_todo);
        Todo {
            id: id,
            task: todo_data.task.clone(),
        }
    }

    async fn get(&self, todo_id: &TodoId) -> Result<Todo, TodoRepoErr> {
        let data = self.unlock().await;
        match data.storage.get(todo_id) {
            Some(persisted) => {
                let todo = Todo {
                    id: todo_id.clone(),
                    task: persisted.task.clone(),
                };
                Ok(todo)
            }
            None => Err(TodoRepoErr::NotFound(*todo_id)),
        }
    }

    async fn list(&self) -> Vec<Todo> {
        let data = self.unlock().await;
        let mut vec: Vec<_> = data
            .storage
            .iter()
            .map(|(id, persisted)| Todo {
                id: *id,
                task: persisted.task.clone(),
            })
            .collect();
        vec.sort_by(|a, b| a.id.cmp(&b.id));
        vec
    }

    async fn delete(&self, todo_id: &TodoId) -> Result<(), TodoRepoErr> {
        let mut data = self.unlock().await;
        match data.storage.remove_entry(todo_id) {
            Some(_) => Ok(()),
            None => Err(TodoRepoErr::NotFound(*todo_id)),
        }
    }

    async fn update(&self, todo: &Todo) -> Result<(), TodoRepoErr> {
        let mut data = self.unlock().await;
        match data.storage.entry(todo.id) {
            Entry::Occupied(mut existing) => {
                existing.insert(PersistedTodo {
                    task: todo.task.clone(),
                });
                Ok(())
            }
            Entry::Vacant(_) => Err(TodoRepoErr::NotFound(todo.id)),
        }
    }
}

struct LastId(u64);

struct PersistedTodo {
    task: String,
}

struct Data {
    last_id: LastId,
    storage: HashMap<TodoId, PersistedTodo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn test_create() {
        let inmem_repo = new();
        let f_create_retrieve = async {
            let to_create = TodoData {
                task: "hello".to_string(),
            };
            let created = inmem_repo.create(&to_create).await;
            let retrieved = inmem_repo.get(&created.id).await;
            retrieved
        };
        match block_on(f_create_retrieve) {
            Ok(retrieved) => assert_eq!("hello".to_string(), retrieved.task),
            _ => panic!("unsuccessful"),
        }
    }

    #[test]
    fn test_get_ok() {
        let inmem_repo = new();
        let created = block_on(async {
            let to_create = TodoData {
                task: "hammertime".to_string(),
            };
            inmem_repo.create(&to_create).await
        });
        let retrieved = block_on(inmem_repo.get(&created.id));
        match retrieved {
            Ok(retrieved) => assert_eq!("hammertime".to_string(), retrieved.task),
            _ => panic!("unsuccessful"),
        }
    }

    #[test]
    fn test_get_not_found() {
        let inmem_repo = new();
        let retrieved = block_on(inmem_repo.get(&TodoId(123131)));
        match retrieved {
            Err(_) => {}
            _ => panic!("unexpectedly found..."),
        }
    }

    #[test]
    fn test_list() {
        let inmem_repo = new();
        let createds = block_on(async {
            let mut createds = Vec::new();
            for i in 0..9 {
                let to_create = TodoData {
                    task: format!("to something {}", i),
                };
                createds.push(inmem_repo.create(&to_create).await);
            }
            createds
        });
        // We could do all of this inside the same `async` block, but this tests
        // that we are doing the right thing across async boundaries
        let listed = block_on(inmem_repo.list());
        assert_eq!(createds, listed);
    }

    #[test]
    fn test_delete_ok() {
        let inmem_repo = new();
        let created = block_on(async {
            let to_create = TodoData {
                task: "hammertime".to_string(),
            };
            inmem_repo.create(&to_create).await
        });
        let deleted = block_on(inmem_repo.delete(&created.id));
        match deleted {
            Ok(_) => {}
            _ => panic!("unsuccessful"),
        }
        let retrieve_after_delete = block_on(inmem_repo.get(&created.id));
        match retrieve_after_delete {
            Err(_) => {}
            _ => panic!("unexpectedly found..."),
        }
    }

    #[test]
    fn test_delete_not_found() {
        let inmem_repo = new();
        let deleted = block_on(inmem_repo.delete(&TodoId(123131)));
        match deleted {
            Err(_) => {}
            _ => panic!("unexpectedly found..."),
        }
    }

    #[test]
    fn test_update_ok() {
        let inmem_repo = new();
        let mut created = block_on(async {
            let to_create = TodoData {
                task: "hammertime".to_string(),
            };
            inmem_repo.create(&to_create).await
        });
        let updated_task = "stop!".to_string();
        created.task = updated_task.clone();
        let updated = block_on(inmem_repo.update(&created));
        match updated {
            Ok(_) => {}
            _ => panic!("unsuccessful"),
        }
        let retrieve_after_update = block_on(inmem_repo.get(&created.id));
        match retrieve_after_update {
            Ok(retrieved) => assert_eq!(updated_task, retrieved.task),
            _ => panic!("unexpectedly found..."),
        }
    }

    #[test]
    fn test_update_not_found() {
        let inmem_repo = new();
        let unpersisted_update = Todo {
            id: TodoId(123213),
            task: "hammertime".to_string(),
        };
        let update = block_on(inmem_repo.update(&unpersisted_update));
        match update {
            Err(_) => {}
            _ => panic!("unexpectedly found..."),
        }
    }
}
