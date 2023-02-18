use std::fs;
use std::env;
use std::io::{self, BufRead};

use std::cell::RefCell;
use std::rc::Rc;


trait Node {
    fn get_name(&self) -> &str;
    fn show(&self, level: usize);
}

struct MyDir {
    name: String,
    children: Vec<Rc<RefCell<ChildType>>>,
    parent: Option<Rc<RefCell<MyDir>>>
}

enum ChildType
{
    Dir(MyDir),
    File(MyFile)
}


impl MyDir {

    fn find(&self, name: &str) -> Option<Rc<RefCell<MyDir>>> {
        match name {
            ".." => {
                if let Some(parent) = &self.parent {
                    return Some(parent.clone())
                }
                None
            },
            "." => None,
            _ => {
                for node in self.children.iter() {

                    let borrowed = node.borrow();
                    match &*borrowed {
                        ChildType::Dir(a) => {
                            if a.name == name {
                                return Some(Rc::new(RefCell::new(*a)))
                            }
                        },
                        _ => {}
                    }
                }
                None
            }
        }
    }

    fn append(&mut self, new_node: Rc<RefCell<ChildType>>) {
        self.children.push(new_node);
    }

}


impl Node for MyDir {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn show(&self, level: usize) {
        for _ in 0..level {
            print!("-");
        }
        println!("dir {}", self.name);
        //for node in self.children.iter() {
        //    node.borrow().show(level + 1);
        //}
    }
}

struct MyFile {
    name: String,
    size: usize,
    parent: Rc<RefCell<MyDir>>
}

impl Node for MyFile {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn show(&self, level: usize) {
        for _ in 0..level {
            print!("-");
        }
        println!("{} {}", self.size, self.name);
    }
}



fn main() {
    let path = env::args().nth(1).expect("No file path was specified.");

    let data = io::BufReader::new(
        fs::File::open(path).expect("Could not open file!")
    ).lines().skip(1);

    // start at the root
    let mut current_node: Rc<RefCell<MyDir>> = Rc::new(
        RefCell::new(
            MyDir {
                name: "/".to_string(),
                children: vec![],
                parent: None
            }
        )
    );

    for line in data.into_iter() {
        let l = line.expect("Could not read line!");
        let (starter, other) = l.split_at(
            l.find(" ").expect(format!("Invalid line \"{}\"!", l).as_str())
        );
        match starter {
            "$" => {  // command, either cd or ls
                let (cmd, params) = other.split_at(
                    other.find(" ").unwrap_or(other.len())
                );
                match cmd {
                    "cd" => {
                        current_node = current_node.borrow().find(params).expect("Invalid cd!");
                    },
                    _ => {}
                }
            },
            "dir" => {  // a directory that's in the current node
                current_node.borrow_mut().append(
                    Rc::new(RefCell::new(
                        ChildType::Dir(MyDir {
                            name: String::from(other),
                            children: vec![],
                            parent: Some(current_node.clone())
                        })
                    ))
                )
            },
            _ => { // a number or a bad starter

            }
        }
    }


    println!("Hello, world!");
}


#[test]
fn test_simple_fs() {
    let root = Rc::new(
        RefCell::new(
            MyDir {
                name: "/".to_string(),
                children: vec![],
                parent: None
            }
        )
    );

    root.borrow_mut().children.push(
        Rc::new(
            RefCell::new(
                ChildType::File(MyFile{
                    name: "afile.exe".to_string(),
                    size: 1000000,
                    parent: Rc::clone(&root)
                })
            )
        )
    );

    root.borrow().show(0);
}
