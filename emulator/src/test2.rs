
struct Foo {}

fn main() {
    let foo = Foo{};
    thread::spawn(move || {
        println!("Here is foo {}", foo);
    })
}