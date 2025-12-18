use crate::io::{framebuffer, line_editor};
use crate::shell;
/// Keyboard input handler with line editing
///
/// This module handles keyboard input events, provides line editing functionality,
/// and echoes input to the framebuffer console.
use fanga_arch_x86_64::keyboard::{KeyCode, KeyEvent};

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
    
    // Alt+F1 through Alt+F12 for virtual terminal switching
    if kbd.is_alt_pressed() {
        match keycode {
            KeyCode::F1 => {
                crate::io::vt::switch_terminal(0);
                return;
            }
            KeyCode::F2 => {
                crate::io::vt::switch_terminal(1);
                return;
            }
            KeyCode::F3 => {
                crate::io::vt::switch_terminal(2);
                return;
            }
            KeyCode::F4 => {
                crate::io::vt::switch_terminal(3);
                return;
            }
            KeyCode::F5 => {
                crate::io::vt::switch_terminal(4);
                return;
            }
            KeyCode::F6 => {
                crate::io::vt::switch_terminal(5);
                return;
            }
            KeyCode::F7 => {
                crate::io::vt::switch_terminal(6);
                return;
            }
            KeyCode::F8 => {
                crate::io::vt::switch_terminal(7);
                return;
            }
            KeyCode::F9 => {
                crate::io::vt::switch_terminal(8);
                return;
            }
            KeyCode::F10 => {
                crate::io::vt::switch_terminal(9);
                return;
            }
            KeyCode::F11 => {
                crate::io::vt::switch_terminal(10);
                return;
            }
            KeyCode::F12 => {
                crate::io::vt::switch_terminal(11);
                return;
            }
            _ => {}
        }
    }
    
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
        KeyCode::Up => {
            // Navigate to previous command in history
            handle_history_prev();
        }
        KeyCode::Down => {
            // Navigate to next command in history
            handle_history_next();
        }
        KeyCode::Tab => {
            // Tab completion
            handle_tab_completion();
        }
        _ => {
            // Ignore other keys
        }
    }
}

/// Redraw the current line in the framebuffer
fn redraw_line(editor: &line_editor::LineEditor) {
    let mut fb = framebuffer::framebuffer();

    // Get prompt length
    let prompt_len = if shell::is_initialized() {
        let shell_guard = shell::shell();
        if let Some(shell) = shell_guard.as_ref() {
            shell.prompt().len()
        } else {
            0
        }
    } else {
        0
    };

    // Redraw the entire line
    fb.redraw_line(0, editor.buffer());

    // Update cursor position
    let row = fb.get_row();
    fb.set_position(prompt_len + editor.cursor(), row);
    fb.draw_cursor();
}

/// Update cursor position without redrawing the whole line
fn update_cursor(editor: &line_editor::LineEditor) {
    let mut fb = framebuffer::framebuffer();

    // Get prompt length
    let prompt_len = if shell::is_initialized() {
        let shell_guard = shell::shell();
        if let Some(shell) = shell_guard.as_ref() {
            shell.prompt().len()
        } else {
            0
        }
    } else {
        0
    };

    // Redraw line to clear old cursor and show new position
    fb.redraw_line(0, editor.buffer());
    let row = fb.get_row();
    fb.set_position(prompt_len + editor.cursor(), row);
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

    // Process the line through the shell
    if shell::is_initialized() {
        // Add to history
        if !line.trim().is_empty() {
            let mut history_guard = shell::history::history();
            if let Some(history) = history_guard.as_mut() {
                history.add(line.clone());
            }
        }

        // Execute the command
        if !line.trim().is_empty() {
            let mut shell_guard = shell::shell();
            if let Some(shell) = shell_guard.as_mut() {
                if let Err(err) = shell.execute(&line) {
                    fb.write_string("Error: ");
                    fb.write_string(err);
                    fb.write_string("\n");
                }
            }
        }

        // Show prompt for next command
        let shell_guard = shell::shell();
        if let Some(shell) = shell_guard.as_ref() {
            if shell.is_running() {
                fb.write_string(shell.prompt());
            }
        }
    }

    // Clear the editor for next line
    let mut editor_guard = line_editor::editor();
    if let Some(editor) = editor_guard.as_mut() {
        editor.clear();
    }
}

/// Handle Up arrow (previous command in history)
fn handle_history_prev() {
    let mut history_guard = shell::history::history();
    if let Some(history) = history_guard.as_mut() {
        if let Some(cmd) = history.prev() {
            // Replace current line with historical command
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                editor.clear();
                for ch in cmd.chars() {
                    editor.insert_char(ch);
                }
                redraw_line(editor);
            }
        }
    }
}

/// Handle Down arrow (next command in history)
fn handle_history_next() {
    let mut history_guard = shell::history::history();
    if let Some(history) = history_guard.as_mut() {
        if let Some(cmd) = history.next() {
            // Replace current line with historical command
            let mut editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_mut() {
                editor.clear();
                for ch in cmd.chars() {
                    editor.insert_char(ch);
                }
                redraw_line(editor);
            }
        }
    }
}

/// Handle Tab key (command completion)
fn handle_tab_completion() {
    // Get current line
    let current_line = {
        let editor_guard = line_editor::editor();
        if let Some(editor) = editor_guard.as_ref() {
            editor.get_line()
        } else {
            return;
        }
    };

    // Only complete if we're at the beginning (completing command name)
    let trimmed = current_line.trim();
    if trimmed.contains(' ') {
        // Already has arguments, don't complete
        return;
    }

    // Try to complete
    if let Some(completed) = shell::completion::complete_single(trimmed) {
        // Single match - complete it
        let mut editor_guard = line_editor::editor();
        if let Some(editor) = editor_guard.as_mut() {
            editor.clear();
            for ch in completed.chars() {
                editor.insert_char(ch);
            }
            redraw_line(editor);
        }
    } else {
        // Multiple or no matches - show possibilities
        let matches = shell::completion::complete(trimmed);
        if !matches.is_empty() {
            let mut fb = framebuffer::framebuffer();
            fb.write_string("\n");
            for cmd in matches {
                fb.write_string(&cmd);
                fb.write_string("  ");
            }
            fb.write_string("\n");

            // Show prompt and current line again
            let shell_guard = shell::shell();
            if let Some(shell) = shell_guard.as_ref() {
                fb.write_string(shell.prompt());
            }

            let editor_guard = line_editor::editor();
            if let Some(editor) = editor_guard.as_ref() {
                for ch in editor.buffer() {
                    let mut s = alloc::string::String::new();
                    s.push(*ch);
                    fb.write_string(&s);
                }
            }
        }
    }
}
