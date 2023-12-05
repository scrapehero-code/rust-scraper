use std::ops::Add;

use scraper::{ElementRef, Html, Selector};

use csv::Writer;

/// Returns the HTML response from sending request to the input URL
fn get_response(url: &String) -> String {
    let resp = reqwest::blocking::get(url).expect("Request failed to start url");
    let body = resp.text().expect("Failed to get the response");
    body
}

/// Gets the `href` attribute of an element and returns the string
fn get_url_from_element(elem: ElementRef) -> String {
    String::from(elem.attr("href").unwrap())
}

/// Extract and returns pokemon URLs from a listing page
fn get_pokemon_urls(listing_page: Html) -> Vec<String> {
    let url_selector = Selector::parse("li.product a.woocommerce-loop-product__link").unwrap();
    let pokemon_urls: Vec<String> = listing_page
        .select(&url_selector)
        .into_iter()
        .map(|elem| get_url_from_element(elem))
        .collect();
    pokemon_urls
}

/// Gets all the texts inside an HTML element
fn get_text_from_element(elem: ElementRef) -> String {
    let mut output_string = String::new();
    for text in elem.text() {
        output_string = output_string.add(text);
    }
    output_string
}

/// Runs css selector using the given selector query on the document and returns one matching element
fn css_select_one(selector: String, document: &Html) -> ElementRef<'_> {
    let selector = Selector::parse(&selector).unwrap();
    let element: ElementRef<'_> = document.select(&selector).next().unwrap();
    element
}

/// Removes any unnecessary whitespaces from the string and returns the cleaned string
fn clean_text(text: String) -> String {
    let x: Vec<&str> = text.split_whitespace().collect();
    x.join(" ")
}
/// Extract and returns a list of required data points from a product page
fn extract_pokemon_data(pokemon_page: Html) -> Vec<String> {
    let title_elem = css_select_one(String::from("h1"), &pokemon_page);
    let title = get_text_from_element(title_elem);

    let price_elem = css_select_one(String::from("p.price span"), &pokemon_page);
    let price = get_text_from_element(price_elem);

    let description_elem = css_select_one(
        String::from("div[class*=\"short-description\"]"),
        &pokemon_page,
    );
    let description = clean_text(get_text_from_element(description_elem));

    let sku_elem = css_select_one(String::from("span.sku"), &pokemon_page);
    let sku = get_text_from_element(sku_elem);

    let stock_count_elem = css_select_one(String::from("p.stock"), &pokemon_page);
    let stock_count_raw = get_text_from_element(stock_count_elem);
    let stock_count = String::from(stock_count_raw.split_whitespace().collect::<Vec<_>>()[0]);

    vec![title, price, description, stock_count, sku]
}

/// Writes the given data to the file
/// 
/// # Parameters
/// * filename - file name to write to
/// * field_names - list of column names of the csv
/// * records - list of records
fn write_to_csv(filename: &str, field_names: Vec<&str>, records: Vec<Vec<String>>) {
    let mut writer = Writer::from_path(filename).unwrap();
    writer.write_record(field_names).unwrap();
    records
        .into_iter()
        .map(|x| writer.write_record(x))
        .for_each(drop);
}

fn main() {
    let start_url = "https://scrapeme.live/shop/";
    let response = get_response(&String::from(start_url));
    let document = Html::parse_document(&response);
    let mut pokemons: Vec<Vec<String>> = Vec::new();

    for url in get_pokemon_urls(document) {
        let pokemon_resp = get_response(&url);
        let pokemon_document = Html::parse_document(&pokemon_resp);
        let pokemon = extract_pokemon_data(pokemon_document);
        pokemons.push(pokemon);
        println!("Processed {}", url);
    }
    let field_names = vec!["title", "price", "sku", "stock_count", "description"];
    write_to_csv("pokemons.csv", field_names, pokemons);
}
