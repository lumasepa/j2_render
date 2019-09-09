use tera::{Tera, Context};
use crate::j2::{filters, functions, testers};

pub fn tera_render(template: String, context: Context) -> String {
    // TODO evaluate lazzy static to avoid recreation of tera
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template)
        .expect("Error loading template in engine");

    tera.register_filter("bash", filters::bash);
    tera.register_filter("sed", filters::sed);
    tera.register_filter("glob", filters::file_glob);
    tera.register_filter("read_file", filters::read_file);
    tera.register_filter("file_name", filters::file_name);
    tera.register_filter("file_dir", filters::file_dir);
    tera.register_filter("strip_line_breaks", filters::strip_line_breaks);
    tera.register_filter("remove_extension", filters::remove_extension);
    tera.register_filter("b64decode", filters::b64decode);
    tera.register_filter("b64encode", filters::b64encode);
    tera.register_filter("str", filters::str);
    tera.register_filter("to_json", filters::str);
    tera.register_filter("from_json", filters::from_json);

    tera.register_function("tab_all_lines", Box::new(functions::tab_all_lines));
    tera.register_function("bash", Box::new(functions::bash));
    tera.register_function("str", Box::new(functions::str));
    tera.register_function("to_json", Box::new(functions::str));
    tera.register_function("from_json", Box::new(functions::from_json));

    tera.register_tester("file", testers::is_file);
    tera.register_tester("directory", testers::is_directory);

    tera.render("template", &context).expect("Error rendering template")
}