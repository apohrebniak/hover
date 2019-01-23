extern crate hover;
use hover::Hover;

fn main() {
    //create an instance of Hover
    //Node is created under the hood
    let mut hover = Hover::new(String::from("127.0.0.1"), 6202);

    //fully blocking start implementation.
    // Node is created and started to run in a separate thread
    hover.start();

    //don't want to join on something
    //letf multicast and connection threads live on theis own
    std::thread::sleep_ms(60000);
}
