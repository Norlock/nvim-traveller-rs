
pub enum VirtTextPos {
    Eol,
    Overlay,
    RightAlign,
    Inline
}

pub enum HlMode {
    Replace,
    Combine,
    Blend
}

pub struct ExtmarkOpts {
    pub id: Option<u32>,
    pub end_row: Option<i32>,
    pub end_col: Option<i32>,
    pub hl_group: Option<String>,
    pub hl_eol: Option<bool>,
    pub virt_text: Option<Vec<(String, String)>>,
    pub virt_text_pos: Option<VirtTextPos>,
    pub virt_text_win_col: Option<u32>,
    pub hl_mode: Option<HlMode>,
    pub virt_lines_above: Option<bool>,
    // TODO more
}
