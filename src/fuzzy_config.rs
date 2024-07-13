use neo_api_rs::{
    mlua::Lua, DummyTask, ExecPreview, ExecStandardSearch, ExecuteTask, FuzzyConfig, FuzzySearch,
    NeoApi, OpenIn, RTM,
};
use std::path::PathBuf;

use crate::state::AppState;

pub struct TravellerFuzzy {
    pub search_type: FuzzySearch,
    pub cwd: PathBuf,
}

impl TravellerFuzzy {
    pub fn new(cwd: PathBuf, search_type: FuzzySearch) -> Self {
        Self { search_type, cwd }
    }
}

impl FuzzyConfig for TravellerFuzzy {
    fn cwd(&self) -> PathBuf {
        self.cwd.clone()
    }

    fn search_type(&self) -> FuzzySearch {
        self.search_type
    }

    // TODO make async
    fn on_enter(&self, lua: &Lua, open_in: OpenIn, selected: PathBuf) {
        match self.search_type {
            FuzzySearch::Directories => RTM.block_on(async move {
                if let Err(err) = AppState::open_navigation(lua, selected).await {
                    let _ = NeoApi::notify(lua, &err);
                }
            }),
            FuzzySearch::Files | FuzzySearch::GitFiles => {
                let _ = NeoApi::open_file(lua, open_in, selected.to_str().unwrap());
            }
            _ => {
                //
            }
        }
    }

    fn search_task(&self, search_query: &str) -> Box<dyn ExecuteTask> {
        let create_standard_tasks = |args: Vec<String>| -> Box<dyn ExecuteTask> {
            Box::new(ExecStandardSearch {
                search_query: search_query.into(),
                cwd: self.cwd(),
                args,
                search_type: self.search_type,
            })
        };

        match self.search_type {
            FuzzySearch::Files | FuzzySearch::GitFiles => {
                let args = vec!["--type".to_string(), "file".to_string()];
                create_standard_tasks(args)
            }
            FuzzySearch::Directories => {
                let args = vec!["--type".to_string(), "directory".to_string()];
                create_standard_tasks(args)
            }
            FuzzySearch::Buffer => Box::new(DummyTask),
        }
    }

    fn preview_task(&self, selected_idx: usize) -> Box<dyn ExecuteTask> {
        Box::new(ExecPreview {
            cwd: self.cwd(),
            selected_idx,
        })
    }
}
