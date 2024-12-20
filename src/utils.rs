pub fn from_tuple_to_html(item: (String, String, String)) -> String {
    let mut str = String::new();
    str.push_str(format!("<a href='{}'>{}€ - {}</a>\n", item.2, item.1, item.0).as_str());
    return str;
}
