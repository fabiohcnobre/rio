use crate::ansi::CursorShape;
use crate::crosswords::grid::row::Row;
use crate::crosswords::pos;
use crate::crosswords::pos::CursorState;
use crate::crosswords::square::{Flags, Square};
use crate::ime::Preedit;
use crate::screen::{constants, context, EventProxy};
use crate::selection::SelectionRange;
use colors::{
    term::{List, TermColors},
    AnsiColor, Colors, NamedColor,
};
use config::Config;
use std::rc::Rc;
use sugarloaf::components::rect::Rect;
use sugarloaf::core::{Sugar, SugarDecoration, SugarStack, SugarStyle};
use sugarloaf::Sugarloaf;

const PADDING_X_TABS: f32 = 30.;

#[derive(Default)]
struct Cursor {
    state: CursorState,
    content: char,
    content_ref: char,
}

pub struct State {
    pub option_as_alt: bool,
    is_ime_enabled: bool,
    named_colors: Colors,
    pub colors: List,
    cursor: Cursor,
    pub selection_range: Option<SelectionRange>,
}

// TODO: Finish from
impl From<Square> for Sugar {
    #[inline]
    fn from(square: Square) -> Sugar {
        let mut style: Option<SugarStyle> = None;
        let is_italic = square.flags.contains(Flags::ITALIC);
        let is_bold_italic = square.flags.contains(Flags::BOLD_ITALIC);
        let is_bold = square.flags.contains(Flags::BOLD);

        if is_bold || is_bold_italic || is_italic {
            style = Some(SugarStyle {
                is_italic,
                is_bold_italic,
                is_bold,
            });
        }

        Sugar {
            content: square.c,
            foreground_color: [0.0, 0.0, 0.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            style,
            decoration: None,
        }
    }
}

impl State {
    pub fn new(config: &Rc<Config>) -> State {
        let term_colors = TermColors::default();
        let colors = List::from(&term_colors);

        let option_as_alt = matches!(
            config.option_as_alt.to_lowercase().as_str(),
            "both" | "left" | "right"
        );

        State {
            option_as_alt,
            is_ime_enabled: false,
            colors,
            selection_range: None,
            named_colors: config.colors,
            cursor: Cursor {
                content: config.cursor,
                content_ref: config.cursor,
                state: CursorState::new(config),
            },
        }
    }

    pub fn get_cursor_state(&self) -> CursorState {
        self.cursor.state.clone()
    }

    // TODO: Square.into()
    #[inline]
    fn create_sugar(&self, square: &Square) -> Sugar {
        let flags = square.flags;

        let foreground_color = match square.fg {
            AnsiColor::Named(NamedColor::Black) => self.named_colors.black,
            AnsiColor::Named(NamedColor::Background) => self.named_colors.background.0,
            AnsiColor::Named(NamedColor::Blue) => self.named_colors.blue,
            AnsiColor::Named(NamedColor::LightBlack) => self.named_colors.light_black,
            AnsiColor::Named(NamedColor::LightBlue) => self.named_colors.light_blue,
            AnsiColor::Named(NamedColor::LightCyan) => self.named_colors.light_cyan,
            AnsiColor::Named(NamedColor::LightForeground) => {
                self.named_colors.light_foreground
            }
            AnsiColor::Named(NamedColor::LightGreen) => self.named_colors.light_green,
            AnsiColor::Named(NamedColor::LightMagenta) => self.named_colors.light_magenta,
            AnsiColor::Named(NamedColor::LightRed) => self.named_colors.light_red,
            AnsiColor::Named(NamedColor::LightWhite) => self.named_colors.light_white,
            AnsiColor::Named(NamedColor::LightYellow) => self.named_colors.light_yellow,
            AnsiColor::Named(NamedColor::Cursor) => self.named_colors.cursor,
            AnsiColor::Named(NamedColor::Cyan) => self.named_colors.cyan,
            AnsiColor::Named(NamedColor::DimBlack) => self.named_colors.dim_black,
            AnsiColor::Named(NamedColor::DimBlue) => self.named_colors.dim_blue,
            AnsiColor::Named(NamedColor::DimCyan) => self.named_colors.dim_cyan,
            AnsiColor::Named(NamedColor::DimForeground) => {
                self.named_colors.dim_foreground
            }
            AnsiColor::Named(NamedColor::DimGreen) => self.named_colors.dim_green,
            AnsiColor::Named(NamedColor::DimMagenta) => self.named_colors.dim_magenta,
            AnsiColor::Named(NamedColor::DimRed) => self.named_colors.dim_red,
            AnsiColor::Named(NamedColor::DimWhite) => self.named_colors.dim_white,
            AnsiColor::Named(NamedColor::DimYellow) => self.named_colors.dim_yellow,
            AnsiColor::Named(NamedColor::Foreground) => self.named_colors.foreground,
            AnsiColor::Named(NamedColor::Green) => self.named_colors.green,
            AnsiColor::Named(NamedColor::Magenta) => self.named_colors.magenta,
            AnsiColor::Named(NamedColor::Red) => self.named_colors.red,
            AnsiColor::Named(NamedColor::White) => self.named_colors.white,
            AnsiColor::Named(NamedColor::Yellow) => self.named_colors.yellow,
            AnsiColor::Spec(rgb) => {
                if !flags.contains(Flags::DIM) {
                    rgb.to_arr()
                } else {
                    rgb.to_arr_with_dim()
                }
            }
            AnsiColor::Indexed(index) => {
                let index = match (flags & Flags::DIM_BOLD, index) {
                    (Flags::DIM, 8..=15) => index as usize - 8,
                    (Flags::DIM, 0..=7) => NamedColor::DimBlack as usize + index as usize,
                    _ => index as usize,
                };

                self.colors[index]
            }
        };

        let background_color = match square.bg {
            AnsiColor::Named(NamedColor::Black) => self.named_colors.black,
            AnsiColor::Named(NamedColor::Background) => self.named_colors.background.0,
            AnsiColor::Named(NamedColor::Blue) => self.named_colors.blue,
            AnsiColor::Named(NamedColor::LightBlack) => self.named_colors.light_black,
            AnsiColor::Named(NamedColor::LightBlue) => self.named_colors.light_blue,
            AnsiColor::Named(NamedColor::LightCyan) => self.named_colors.light_cyan,
            AnsiColor::Named(NamedColor::LightForeground) => {
                self.named_colors.light_foreground
            }
            AnsiColor::Named(NamedColor::LightGreen) => self.named_colors.light_green,
            AnsiColor::Named(NamedColor::LightMagenta) => self.named_colors.light_magenta,
            AnsiColor::Named(NamedColor::LightRed) => self.named_colors.light_red,
            AnsiColor::Named(NamedColor::LightWhite) => self.named_colors.light_white,
            AnsiColor::Named(NamedColor::LightYellow) => self.named_colors.light_yellow,
            AnsiColor::Named(NamedColor::Cursor) => self.named_colors.cursor,
            AnsiColor::Named(NamedColor::Cyan) => self.named_colors.cyan,
            AnsiColor::Named(NamedColor::DimBlack) => self.named_colors.dim_black,
            AnsiColor::Named(NamedColor::DimBlue) => self.named_colors.dim_blue,
            AnsiColor::Named(NamedColor::DimCyan) => self.named_colors.dim_cyan,
            AnsiColor::Named(NamedColor::DimForeground) => {
                self.named_colors.dim_foreground
            }
            AnsiColor::Named(NamedColor::DimGreen) => self.named_colors.dim_green,
            AnsiColor::Named(NamedColor::DimMagenta) => self.named_colors.dim_magenta,
            AnsiColor::Named(NamedColor::DimRed) => self.named_colors.dim_red,
            AnsiColor::Named(NamedColor::DimWhite) => self.named_colors.dim_white,
            AnsiColor::Named(NamedColor::DimYellow) => self.named_colors.dim_yellow,
            AnsiColor::Named(NamedColor::Foreground) => self.named_colors.foreground,
            AnsiColor::Named(NamedColor::Green) => self.named_colors.green,
            AnsiColor::Named(NamedColor::Magenta) => self.named_colors.magenta,
            AnsiColor::Named(NamedColor::Red) => self.named_colors.red,
            AnsiColor::Named(NamedColor::White) => self.named_colors.white,
            AnsiColor::Named(NamedColor::Yellow) => self.named_colors.yellow,
            AnsiColor::Spec(rgb) => rgb.to_arr(),
            AnsiColor::Indexed(idx) => self.colors[idx as usize],
        };

        let content = if square.c == '\t' || flags.contains(Flags::HIDDEN) {
            ' '
        } else {
            square.c
        };

        let mut style: Option<SugarStyle> = None;
        let is_italic = flags.contains(Flags::ITALIC);
        let is_bold_italic = flags.contains(Flags::BOLD_ITALIC);
        let is_bold = flags.contains(Flags::BOLD);

        if is_bold || is_bold_italic || is_italic {
            style = Some(SugarStyle {
                is_italic,
                is_bold_italic,
                is_bold,
            });
        }

        let mut decoration = None;
        if flags.contains(Flags::UNDERLINE) {
            decoration = Some(SugarDecoration {
                position: (0.0, 0.92),
                size: (1.0, 0.025),
                color: self.named_colors.foreground,
            });
        } else if flags.contains(Flags::STRIKEOUT) {
            decoration = Some(SugarDecoration {
                position: (0.0, 0.5),
                size: (1.0, 0.025),
                color: self.named_colors.foreground,
            });
        }

        Sugar {
            content,
            foreground_color,
            background_color,
            style,
            decoration,
        }
    }

    #[inline]
    fn cursor_to_decoration(&self) -> Option<SugarDecoration> {
        match self.cursor.state.content {
            CursorShape::Block => Some(SugarDecoration {
                position: (0.0, 0.0),
                size: (1.0, 1.0),
                color: self.named_colors.cursor,
            }),
            CursorShape::Underline => Some(SugarDecoration {
                position: (0.0, 0.92),
                size: (1.0, 0.05),
                color: self.named_colors.cursor,
            }),
            CursorShape::Beam => Some(SugarDecoration {
                position: (0.0, 0.0),
                size: (0.1, 1.0),
                color: self.named_colors.cursor,
            }),
            CursorShape::Hidden => None,
        }
    }

    #[inline]
    fn create_empty_sugar_stack_from_columns(&self, columns: usize) -> SugarStack {
        let mut stack: Vec<Sugar> = vec![];
        for _ in 0..columns {
            stack.push(Sugar {
                content: ' ',
                foreground_color: self.named_colors.background.0,
                background_color: self.named_colors.background.0,
                style: None,
                decoration: None,
            })
        }
        stack
    }

    #[inline]
    fn create_sugar_stack_with_selection(
        &mut self,
        row: &Row<Square>,
        has_cursor: bool,
        range: &SelectionRange,
        line: pos::Line,
        display_offset: i32,
    ) -> SugarStack {
        let mut stack: Vec<Sugar> = vec![];
        let columns: usize = row.len();
        for column in 0..columns {
            let line = line - display_offset;
            let is_selected = range.contains(pos::Pos::new(line, pos::Column(column)));
            let square = &row.inner[column];
            if has_cursor && column == self.cursor.state.pos.col {
                stack.push(self.create_cursor(square));
            } else if is_selected {
                let content = if square.c == '\t' || square.flags.contains(Flags::HIDDEN)
                {
                    ' '
                } else {
                    square.c
                };

                let selected_sugar = Sugar {
                    content,
                    foreground_color: self.named_colors.background.0,
                    background_color: self.named_colors.light_blue,
                    style: None,
                    decoration: None,
                };
                stack.push(selected_sugar);
            } else {
                stack.push(self.create_sugar(square));
            }

            // Render last column and break row
            if column == (columns - 1) {
                break;
            }
        }

        stack
    }

    #[inline]
    fn create_sugar_stack(&mut self, row: &Row<Square>, has_cursor: bool) -> SugarStack {
        let mut stack: Vec<Sugar> = vec![];
        let columns: usize = row.len();
        for column in 0..columns {
            let square = &row.inner[column];

            if has_cursor && column == self.cursor.state.pos.col {
                stack.push(self.create_cursor(square));
            } else {
                stack.push(self.create_sugar(square));
            }

            // Render last column and break row
            if column == (columns - 1) {
                break;
            }
        }

        stack
    }

    #[inline]
    fn create_cursor(&self, square: &Square) -> Sugar {
        let mut cloned_square = square.clone();

        // If IME is enabled we get the current content to cursor
        if self.is_ime_enabled {
            cloned_square.c = self.cursor.content;
        }

        // If IME is enabled or is a block cursor, put background color
        // when cursor is over the character
        if self.is_ime_enabled || self.cursor.state.content == CursorShape::Block {
            cloned_square.fg = AnsiColor::Named(NamedColor::Background);
        }

        let mut sugar = self.create_sugar(&cloned_square);
        sugar.decoration = self.cursor_to_decoration();
        sugar
    }

    pub fn set_ime(&mut self, ime_preedit: Option<&Preedit>) {
        if let Some(preedit) = ime_preedit {
            if let Some(content) = preedit.text.chars().next() {
                self.cursor.content = content;
                self.is_ime_enabled = true;
                return;
            }
        }

        self.is_ime_enabled = false;
        self.cursor.content = self.cursor.content_ref;
    }

    #[inline]
    pub fn set_selection(&mut self, selection_range: Option<SelectionRange>) {
        self.selection_range = selection_range;
    }

    #[inline]
    pub fn update(
        &mut self,
        rows: Vec<Row<Square>>,
        cursor: CursorState,
        sugarloaf: &mut Sugarloaf,
        context_manager: &context::ContextManager<EventProxy>,
        display_offset: i32,
    ) {
        self.cursor.state = cursor;
        let is_cursor_visible = self.cursor.state.is_visible();

        if let Some(active_selection) = self.selection_range {
            for (i, row) in rows.iter().enumerate() {
                let has_cursor = is_cursor_visible && self.cursor.state.pos.row == i;
                let sugar_stack = self.create_sugar_stack_with_selection(
                    row,
                    has_cursor,
                    &active_selection,
                    pos::Line(i as i32),
                    display_offset,
                );
                sugarloaf.stack(sugar_stack);
            }
        } else {
            for (i, row) in rows.iter().enumerate() {
                let has_cursor = is_cursor_visible && self.cursor.state.pos.row == i;
                let sugar_stack = self.create_sugar_stack(row, has_cursor);
                sugarloaf.stack(sugar_stack);
            }
        }

        // This is a fake row created only for visual purposes
        let empty_last_line =
            self.create_empty_sugar_stack_from_columns(sugarloaf.layout.columns);
        sugarloaf.stack(empty_last_line);

        if context_manager.len() > 1 {
            let mut renderable_tabs = vec![];
            let mut initial_position =
                (sugarloaf.layout.width / sugarloaf.layout.scale_factor) - PADDING_X_TABS;
            let position_modifier = 20.;
            for (i, _) in context_manager.contexts().iter().enumerate() {
                let mut color = self.named_colors.tabs;
                let mut size = constants::INACTIVE_TAB_WIDTH_SIZE;
                if i == context_manager.current_index() {
                    color = self.named_colors.tabs_active;
                    size = constants::ACTIVE_TAB_WIDTH_SIZE;
                }
                let renderable = Rect {
                    position: [initial_position, 0.0],
                    color,
                    size: [30.0, size],
                };
                initial_position -= position_modifier;
                renderable_tabs.push(renderable);
            }
            sugarloaf.pile_rect(renderable_tabs);
        }
    }
}
