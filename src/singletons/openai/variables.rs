use crate::USERNAME;

pub fn transform_variables(input: &str) -> String {
    let mut owned = input.to_owned();

    let now = chrono::Local::now().format("%A, %B %d, %Y at %I:%M %p %Z").to_string(); 

    owned = owned.replace("{USERNAME}", &USERNAME);
    owned = owned.replace("{DATETIME}", &now);

    owned
}