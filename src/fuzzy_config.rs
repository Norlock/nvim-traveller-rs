use neo_api_rs::{
    mlua::Lua, BufferSearch, ExecDirectorySearch, ExecFileSearch, ExecPreview,
    ExecRecentDirectories, ExecuteTask, FuzzyConfig, FuzzySearch, NeoApi, NeoDebug, NeoUtils,
    OpenIn, RTM,
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
                    NeoDebug::log(err).await;
                }
            }),
            FuzzySearch::Files | FuzzySearch::GitFiles | FuzzySearch::Buffer => {
                if let Err(_e) =
                    NeoApi::open_file(lua, open_in, selected.to_string_lossy().as_ref())
                {
                    // TODO
                }
            }
        }
    }

    fn search_task(
        &self,
        lua: &Lua,
        search_query: String,
        selected_tab: usize,
    ) -> Box<dyn ExecuteTask> {
        match self.search_type {
            FuzzySearch::Files => Box::new(ExecFileSearch {
                cmd: "fd",
                search_query,
                cwd: self.cwd(),
                args: vec!["--type", "file"],
            }),
            FuzzySearch::GitFiles => Box::new(ExecFileSearch {
                cmd: "git",
                search_query,
                cwd: self.cwd(),
                args: vec!["ls-files", "--cached", "--others", "--exclude-standard"],
            }),
            FuzzySearch::Directories => {
                if selected_tab == 0 {
                    Box::new(ExecDirectorySearch {
                        cmd: "fd",
                        search_query,
                        cwd: NeoUtils::home_directory(),
                        args: vec!["--type", "directory"],
                    })
                } else {
                    Box::new(ExecRecentDirectories::new(search_query))
                }
            }
            FuzzySearch::Buffer => {
                Box::new(BufferSearch::new(lua, &self.cwd(), selected_tab).unwrap())
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
}
