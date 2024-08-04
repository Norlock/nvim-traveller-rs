use neo_api_rs::{
    mlua::Lua, BufInfoOpts, BufferSearch, ExecPreview, ExecRecentDirectories, ExecStandardSearch,
    ExecuteTask, FuzzyConfig, FuzzySearch, NeoApi, OpenIn, RTM,
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
            FuzzySearch::Files | FuzzySearch::GitFiles | FuzzySearch::Buffer => {
                let _ = NeoApi::open_file(lua, open_in, selected.to_str().unwrap());
            }
        }
    }

    fn search_task(&self, lua: &Lua, search_query: String, tab_idx: usize) -> Box<dyn ExecuteTask> {
        match self.search_type {
            FuzzySearch::Files => Box::new(ExecStandardSearch {
                cmd: "fd",
                search_query,
                cwd: self.cwd(),
                args: vec!["--type", "file"],

                search_type: self.search_type,
            }),
            FuzzySearch::GitFiles => Box::new(ExecStandardSearch {
                cmd: "git",
                search_query,
                cwd: self.cwd(),
                args: vec!["ls-files", "--cached", "--others", "--exclude-standard"],
                search_type: self.search_type,
            }),
            FuzzySearch::Directories => {
                if tab_idx == 0 {
                    Box::new(ExecStandardSearch {
                        cmd: "fd",
                        search_query,
                        cwd: self.cwd(),
                        args: vec!["--type", "directory"],
                        search_type: self.search_type,
                    })
                } else {
                    Box::new(ExecRecentDirectories::new(lua, search_query).unwrap())
                }
            }
            FuzzySearch::Buffer => {
                let buf_infos = NeoApi::get_buf_info(lua, BufInfoOpts::BufListed)
                    .expect("Buf info not working");

                Box::new(BufferSearch {
                    cwd: self.cwd(),
                    search_query,
                    buf_infos,
                })
            }
        }
    }

    fn preview_task(
        &self,
        _lua: &Lua,
        selected_idx: usize,
        _tab_idx: usize,
    ) -> Box<dyn ExecuteTask> {
        Box::new(ExecPreview {
            cwd: self.cwd(),
            selected_idx,
        })
    }

    fn tabs(&self) -> Vec<Box<str>> {
        match self.search_type {
            FuzzySearch::Directories => {
                vec![" All directories ".into(), " Last used ".into()]
            }
            _ => vec![],
        }
    }
}
