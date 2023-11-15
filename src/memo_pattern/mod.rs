struct Document{
    data: String,
}

impl Document{
    pub fn new() -> Self{
        Self { data:  String::new()}
    }

    pub fn set_document(&mut self,content:String){
        self.data = content;
    }
    
    pub fn clear_docunment(&mut self){
        self.data.clear();
    }

    pub fn create_memento(&self) -> Memento{
        Memento{
            data: self.data.clone()
        }
    }

    pub fn document_from_memento(&mut self,memo : &Memento){
        self.data = memo.data.clone();
    }
}

struct Memento{
    data : String
}

struct Caretaker{
    saved_states: Vec<Memento>,
}

impl  Caretaker {
    fn new() -> Caretaker{
        Caretaker { saved_states: vec![] }
    }

    fn save(&mut self,memento:Memento){
        self.saved_states.push(memento);
    }

    fn undo(&mut self)-> Option<Memento>{
        self.saved_states.pop()
    }
}


#[test]
fn main() {
    let mut doc = Document::new();
    let mut caretaker = Caretaker::new();

    // 修改并保存文档状态
    doc.set_document("This is the first sentence.".to_string());
    caretaker.save(doc.create_memento());

    // 再次修改并保存文档状态
    doc.set_document("This is the second sentence.".to_string());
    caretaker.save(doc.create_memento());

    // 撤销到上一个状态
    if let Some(memento) = caretaker.undo() {
        doc.document_from_memento(&memento);
        println!("Undo: {}", doc.data);
    }

    // 再次撤销到更早的状态
    if let Some(memento) = caretaker.undo() {
        doc.document_from_memento(&memento);
        println!("Undo: {}", doc.data);
    }
}
