use application::Application;

pub fn move_up(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_up();
}

pub fn move_down(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_down();
}

pub fn move_left(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_left();
}

pub fn move_right(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_right();
}

pub fn move_to_start_of_line(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_to_start_of_line();
}

pub fn move_to_end_of_line(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().cursor.move_to_end_of_line();
}
