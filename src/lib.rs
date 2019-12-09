#![warn(clippy::all)]
use wasm_bindgen::prelude::*;
use web_sys;
use web_sys::Event;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use std::panic;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn get_window() -> web_sys::Window {
    web_sys::window().expect("can't get window")
}

const QUESTIONS: [&str; 20] = [
    "Quel est ce symbole ?<br><img src=\"q0.png\" alt=\"image porte logique\" draggable=false>",
    "Et celui ci ?<br><img src=\"q1.png\" alt=\"image porte logique\" draggable=false>",
    "Encore un dernier :<br><img src=\"q2.png\" alt=\"image porte logique\" draggable=false>",
    "A quoi correspond cette table de vérité ?<br><img src=\"q3.png\" alt=\"image table de vérité\" draggable=false>",
    "A quoi correspond cette autre table de vérité ?<br><img src=\"q4.png\" alt=\"image table de vérité\" draggable=false>",
    "A quoi correspond cette dernière table de vérité ?<br><img src=\"q5.png\" alt=\"image table de vérité\" draggable=false>",
    "Considérez ce circuit logique :<br><img src=\"q6-7-8-9.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 0, B = 1 et Cin = 0 ?<br>",
    "Considérez de nouveau ce circuit logique :<br><img src=\"q6-7-8-9.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 1, B = 1 et Cin = 1 ?<br>",
    "Considérez encore ce circuit logique :<br><img src=\"q6-7-8-9.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 1, B = 0 et Cin = 1 ?<br>",
    "Considérez une dernière fois cce circuit logique :<br><img src=\"q6-7-8-9.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 0, B = 0 et Cin = 1 ?<br>",
    "Combien de bits contiennent les octets ?",
    "Quelle est l'utilité de ce circuit logique ?<br><img src=\"q11.jpg\" alt=\"image circuit logique\" draggable=false>",
    "Considérez ce circuit logique :<br><img src=\"q12-13-14-15.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 0 et B = 0 ?<br>",
    "Considérez de nouveau ce circuit logique :<br><img src=\"q12-13-14-15.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 0 et B = 1 ?<br>",
    "Considérez encore ce circuit logique :<br><img src=\"q12-13-14-15.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 1 et B = 0 ?<br>",
    "Considérez une dernière fois ce circuit logique :<br><img src=\"q12-13-14-15.png\" alt=\"circuit logique\" draggable=false><br>Qu'obtient t'on avec A = 1 et B = 1 ?<br>",
    "Que fait l'instruction d'assembleur \"ADD R0, R1, #42\" ?",
    "Que fait l'instruction d'assembleur \"STR R3, 654\"",
    "Que fait l'instruction d'assembleur \"MOV R1, #42\"",
    "Que fait l'instruction d'assembleur \"HALT\"",
];

const ANSWERS: [(&str, &str, &str, &str); 20] = [
    ("OU EXCLUSIF", "ET", "OU", "NON"), // identification des portes
    ("OU EXCLUSIF", "ET", "OU", "NON"),
    ("OU EXCLUSIF", "ET", "OU", "NON"),
    ("OU EXCLUSIF", "ET", "OU", "NON"), // identification des tables de vérité
    ("OU EXCLUSIF", "ET", "OU", "NON"),
    ("OU EXCLUSIF", "ET", "OU", "NON"),
    ("A = 0, B = 0", "A = 0, B = 1", "A = 1, B = 0", "A = 1, B = 1"), // additionneur
    ("A = 0, B = 0", "A = 0, B = 1", "A = 1, B = 0", "A = 1, B = 1"),
    ("A = 0, B = 0", "A = 0, B = 1", "A = 1, B = 0", "A = 1, B = 1"),
    ("A = 0, B = 0", "A = 0, B = 1", "A = 1, B = 0", "A = 1, B = 1"),
    ("2", "4", "8", "16"), // octet
    ("Comparer deux bits", "Stocker un bit", "Additionner 2 bits", "Autre chose"),
    ("C = 0, D = 0", "C = 0, D = 1", "C = 1, D = 0", "C = 1, D = 1"),
    ("C = 0, D = 0", "C = 0, D = 1", "C = 1, D = 0", "C = 1, D = 1"),
    ("C = 0, D = 0", "C = 0, D = 1", "C = 1, D = 0", "C = 1, D = 1"),
    ("C = 0, D = 0", "C = 0, D = 1", "C = 1, D = 0", "C = 1, D = 1"),
    ("Additionne R0 et 42 et place le résultat dans R1", "Additionne R1 et 42 et place le résultat dans R0", "Additionne R1 et R0 et place le résultat à l'adresse 42 de la RAM", "Compare la valeur de R1 + R0 à 42"),
    ("Place (entrepose) la valeur de R3 dans la RAM, à l'adresse 654", "Charge la valeur à l'adresse 654 de la RAM dans R3", "Soustrait 654 à R3", "Compare R3 à 654"),
    ("Ajoute 42 à R1", "Place la valeur de R1 dans la RAM, à l'adresse 42", "Donne la valeur du sens de la vie (42) à R1", "Multiplie R1 par 42"),
    ("Déclenche une interruption", "Eteint la machine", "Remet à 0 tous les registres", "Arrête l'exécution du programme")
];

const GOOD_ANSWERS: [usize; 20] = [
    2, // identification des portes
    3,
    4,
    1, // identification des tables de vérité
    2,
    3,
    3, // additionneur
    4,
    2,
    3,
    3, // octet
    2,
    4, // logic.ly
    1,
    4,
    4,
    2, // asm
    1,
    3,
    4
];

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Program started, wasm is working.");
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let page: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let note: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let window = get_window();
    let document = window.document().expect("can't get document");
    
    let main_element = document
        .create_element("div").expect("can't create div element")
        .dyn_into::<web_sys::HtmlDivElement>().expect("can't cast div element");
    main_element.set_inner_html("Le questionnaire comporte 20 questions et aboutira à une note sur 20.<br><br>
        REGLES :<br>
        <ol>
            <li>Ne jamais quitter l'onglet (dès maintenant). La question serait passée. <span class=\"important\">ATTENTION: Sélectionner et glisser un texte ou glisser une image est considéré par le navigateur comme une sortie de l'onglet; ne vous faîtes pas avoir.</span></li>
            <li>Ne pas relancer le questionnaire une deuxième fois. Vous subiriez une sanction indéfinie.</li>
            <li>Ne pas tenter de modifier sa note à la fin du questionnaire. Vous subiriez une sanction indéfinie.</li>
            <li>Ne pas tenter de trouver les réponses dans le programme. Vous perdriez un temps fou en vain.</li>
        </ol>
        
        <br>
        Un anticheat s'assurera que vous respectez les règles.<br>
        Bonne chance");
    document.body().expect("can't get body").append_child(&main_element).expect("can't append div to body");
    let button = Rc::new(document
        .create_element("button").expect("can't create button")
        .dyn_into::<web_sys::HtmlButtonElement>().expect("can't cast button"));
    button.set_inner_text("Suivant");
    main_element.append_child(&button).expect("can't append button to div");
    get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "black").expect("can't set class to black");
    let storage = window.local_storage().expect("can't get storage").expect("no storage");

    let br1 = Rc::new(document
        .create_element("br").expect("can't create br element")
        .dyn_into::<web_sys::HtmlBrElement>().expect("can't cast element"));
    let br2 = Rc::new(document
        .create_element("br").expect("can't create br element")
        .dyn_into::<web_sys::HtmlBrElement>().expect("can't cast element"));
    let br3 = Rc::new(document
        .create_element("br").expect("can't create br element")
        .dyn_into::<web_sys::HtmlBrElement>().expect("can't cast element"));
    let br4 = Rc::new(document
        .create_element("br").expect("can't create br element")
        .dyn_into::<web_sys::HtmlBrElement>().expect("can't cast element"));
    let input1 = document
        .create_element("input").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlInputElement>().expect("can't cast element");
    let input2 = document
        .create_element("input").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlInputElement>().expect("can't cast element");
    let input3 = document
        .create_element("input").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlInputElement>().expect("can't cast element");
    let input4 = document
        .create_element("input").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlInputElement>().expect("can't cast element");
    let label1 = document
        .create_element("label").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlLabelElement>().expect("can't cast element");
    let label2 = document
        .create_element("label").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlLabelElement>().expect("can't cast element");
    let label3 = document
        .create_element("label").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlLabelElement>().expect("can't cast element");
    let label4 = document
        .create_element("label").expect("can't create input or label element")
        .dyn_into::<web_sys::HtmlLabelElement>().expect("can't cast element");
    input1.set_attribute("type", "radio").expect("can't set attribute");
    input2.set_attribute("type", "radio").expect("can't set attribute");
    input3.set_attribute("type", "radio").expect("can't set attribute");
    input4.set_attribute("type", "radio").expect("can't set attribute");
    input1.set_attribute("name", "answer").expect("can't set attribute");
    input2.set_attribute("name", "answer").expect("can't set attribute");
    input3.set_attribute("name", "answer").expect("can't set attribute");
    input4.set_attribute("name", "answer").expect("can't set attribute");
    input1.set_attribute("id", "input1").expect("can't set attribute");
    input2.set_attribute("id", "input2").expect("can't set attribute");
    input3.set_attribute("id", "input3").expect("can't set attribute");
    input4.set_attribute("id", "input4").expect("can't set attribute");
    label1.set_attribute("for", "input1").expect("can't set attribute");
    label2.set_attribute("for", "input2").expect("can't set attribute");
    label3.set_attribute("for", "input3").expect("can't set attribute");
    label4.set_attribute("for", "input4").expect("can't set attribute");
    
    let button2 = Rc::clone(&button);
    let page2 = Rc::clone(&page);
    let note2 = Rc::clone(&note);
    let suivant = Closure::wrap(Box::new(move |_event: Event| {
        let mut page = page2.borrow_mut();
        let mut note = note2.borrow_mut();
        
        if *page >= 1 && *page <= 20 && (GOOD_ANSWERS[*page - 1] == 1 && input1.checked() || GOOD_ANSWERS[*page - 1] == 2 && input2.checked() || GOOD_ANSWERS[*page - 1] == 3 && input3.checked() || GOOD_ANSWERS[*page - 1] == 4 && input4.checked()) {
            *note += 1;
        }
        console_log!("zeez");

        *page += 1;
        match *page {
            1...20 => {
                let counter: usize = storage.get_item("counter").expect("can't use storage").unwrap_or(String::from("0")).parse().unwrap_or(0);
                storage.set_item("counter", &format!("{}", counter+1)).expect("can't use storage");
                get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "white").expect("can't set class to white");
                main_element.set_inner_html(QUESTIONS[*page-1]);
                label1.set_inner_text(ANSWERS[*page-1].0);
                label2.set_inner_text(ANSWERS[*page-1].1);
                label3.set_inner_text(ANSWERS[*page-1].2);
                label4.set_inner_text(ANSWERS[*page-1].3);
                input1.set_checked(false);
                input2.set_checked(false);
                input3.set_checked(false);
                input4.set_checked(false);
                main_element.append_child(&br1).expect("can't append child (br or input or label)");
                main_element.append_child(&input1).expect("can't append child (br or input or label)");
                main_element.append_child(&label1).expect("can't append child (br or input or label)");
                main_element.append_child(&br2).expect("can't append child (br or input or label)");
                main_element.append_child(&input2).expect("can't append child (br or input or label)");
                main_element.append_child(&label2).expect("can't append child (br or input or label)");
                main_element.append_child(&br3).expect("can't append child (br or input or label)");
                main_element.append_child(&input3).expect("can't append child (br or input or label)");
                main_element.append_child(&label3).expect("can't append child (br or input or label)");
                main_element.append_child(&br4).expect("can't append child (br or input or label)");
                main_element.append_child(&input4).expect("can't append child (br or input or label)");
                main_element.append_child(&label4).expect("can't append child (br or input or label)");
            },
            _ => {
                console_log!("note = {}", note);
                storage.set_item("note", &format!("{}", note)).expect("can't use storage to set note");
                let counter: usize = storage.get_item("counter").expect("can't use storage").unwrap_or(String::from("0")).parse().unwrap_or(0);
                if counter == 20 {
                    get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "green").expect("can't set class to blue or green");
                } else {
                    get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "blue").expect("can't set class to blue or green");
                }
                
                main_element.set_inner_text(&format!("Votre note est {}/20", note));

                let main_element = main_element.clone();
                let initial_value = format!("Votre note est {}/20", note);
                let anticheat_loop = Closure::wrap(Box::new(move || {
                    let data = main_element.inner_text();
                    
                    if data != initial_value {
                        console_log!("data changed! {} -> {}", initial_value, data);
                        get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "red").expect("can't set class to red");
                    }
                }) as Box<dyn FnMut()>);
            
                get_window()
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        anticheat_loop.as_ref().unchecked_ref(),
                        100,
                    )
                    .expect("Can't launch anticheat_loop loop");
                anticheat_loop.forget();
            }
        }
        if *page < 21 {
            main_element.append_child(&button2).expect("can't append button");
        }
    }) as Box<dyn FnMut(Event)>);
    let button3 = Rc::clone(&button);
    let blur = Closure::wrap(Box::new(move |_event: Event| {
        get_window().document().expect("can't get document").body().expect("can't get body").set_attribute("class", "yellow").expect("can't set class to yellow");
        
        get_window().alert_with_message("Vous avez quitté l'onglet. La question a été passée. (Sélectionner et glisser un texte est considéré par le navigateur comme une sortie d'onglet.)");
        button3.click();
    }) as Box<dyn FnMut(Event)>);
    window
        .add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())
        .expect("can't add event listener");
    button
        .add_event_listener_with_callback("click", suivant.as_ref().unchecked_ref())
        .expect("can't add event listener");
    blur.forget();
    suivant.forget();
}

