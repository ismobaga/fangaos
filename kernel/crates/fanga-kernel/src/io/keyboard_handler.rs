/// Keyboard input handler with line editing
///
/// This module handles keyboard input events, provides line editing functionality,
/// and echoes input to the framebuffer console.

use fanga_arch_x86_64::keyboard::{KeyCode, KeyEvent};
use crate::io::{framebuffer, line_editor};

/// Handle a keyboard event
pub fn handle_key_event(event: KeyEvent, kbd: &fanga_arch_x86_64::keyboard::Keyboard) {
    // Only handle key presses
    if let KeyEvent::Press(keycode) = event {
        handle_key_press(keycode, kbd);
    }
}

/// Handle a key press event
fn handle_key_press(keycode: KeyCode, kbd: &fanga_arch_x86_64::keyboard::Keyboard) {
    // Handle special key combinations first
    if kbd.is_ctrl_pressed() {
        match keycode {
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Ctrl+C - interrupt current line
                handle_ctrl_c();
                return;
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Ctrl+D - EOF signal
                handle_ctrl_d();
                return;
            }
            _ => {}
        }
    }

    // Handle regular keys
    match keycode {
        KeyCode::Char(_ch) => {
            // Insert character
            if let Some(ascii) = kbd.to_ascii(keycode) {
                let mut editor_guard = line_editor::editor();
                if let Some(editor) = editor_guard.as_mut() {
                    if editor.insert_char(ascii) {
                        redraw_line(editor);
                    }
                }
            }
        }
        KeyCode::Backspace => {
            // Handle backspace
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.backspace() {
                    redraw_line(editor);
                }
            }
        }
        KeyCode::Delete => {
            // Handle delete
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.delete_char() {
                    redraw_line(editor);
                }
            }
        }
        KeyCode::Left => {
            // Move cursor left
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.move_left() {
                    update_cursor(editor);
                }
            }
        }
        KeyCode::Right => {
            // Move cursor right
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.move_right() {
                    update_cursor(editor);
                }
            }
        }
        KeyCode::Home => {
            // Move to beginning of line
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.move_home() {
                    update_cursor(editor);
                }
            }
        }
        KeyCode::End => {
            // Move to end of line
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.move_end() {
                    update_cursor(editor);
                }
            }
        }
        KeyCode::Enter => {
            // Submit the line
            handle_enter();
        }
        _ => {
            // Ignore other keys
        }
    }
}

/// Redraw the current line in the framebuffer
fn redraw_line(editor: &line_editor::LineEditor) {
    let mut fb = framebuffer::framebuffer();
    let start_col = 0; // For now, assume lines start at column 0
    
    // Redraw the entire line
    fb.redraw_line(start_col, editor.buffer());
    
    // Update cursor position
    let row = fb.get_row();
    fb.set_position(start_col + editor.cursor(), row);
    fb.draw_cursor();
}

/// Update cursor position without redrawing the whole line
fn update_cursor(editor: &line_editor::LineEditor) {
    let mut fb = framebuffer::framebuffer();
    let start_col = 0;
    
    // Redraw line to clear old cursor and show new position
    fb.redraw_line(start_col, editor.buffer());
    let row = fb.get_row();
    fb.set_position(start_col + editor.cursor(), row);
    fb.draw_cursor();
}

/// Handle Ctrl+C (interrupt)
fn handle_ctrl_c() {
    let mut fb = framebuffer::framebuffer();
    fb.write_string("^C\n");
    
    // Clear the line editor
    let mut editor_guard = line_editor::editor();
    if let Some(editor) = editor_guard.as_mut() {
        editor.clear();
    }
}

/// Handle Ctrl+D (EOF)
fn handle_ctrl_d() {
    let mut fb = framebuffer::framebuffer();
    let editor_guard = line_editor::editor();
    
    if let Some(editor) = editor_guard.as_ref() {
        if editor.is_empty() {
            // On empty line, Ctrl+D shows EOF
            fb.write_string("^D\n");
        } else {
            // On non-empty line, delete character at cursor (same as Delete key)
            drop(editor_guard);
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                if editor.delete_char() {
                    redraw_line(editor);
                }
            }
        }
    }
}

/// Handle Enter key (submit line)
fn handle_enter() {
    let mut fb = framebuffer::framebuffer();
    
    // Get the completed line
    let line = {
        let editor_guard = line_editor::editor();
        if let Some(editor) = editor_guard.as_ref() {
            editor.get_line()
        } else {
            alloc::string::String::new()
        }
    };
    
    // Echo newline
    fb.write_string("\n");
    
    // Process the line (for now, just echo it back)
    if !line.is_empty() {
        fb.write_string("You entered: ");
        fb.write_string(&line);
        fb.write_string("\n");
    }
    
    // Clear the editor for next line
    let mut editor_guard = line_editor::editor();
    if let Some(editor) = editor_guard.as_mut() {
        editor.clear();
    }
}
