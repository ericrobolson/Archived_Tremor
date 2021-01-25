mod ecs;
mod math{
    pub use game_math::f32::*;
}

pub enum MultiplayerMode{
    DeterministicRollback,
    ClientServer
}

pub type ClientId = u32;

pub struct Server {
    clients: Vec<Client>,
    max_outgoing_packet_bytes: usize,
    max_clients: u32,
    outbound_tick_rate: u32,
}
impl Server {
    pub fn main_loop(&mut self) {
        loop {
            // Receive network messages
            self.inbound_network();
            // Handle network messages
            self.tick();
            // Send network messages
            self.outbound_network();
        }
    }

    pub fn inbound_network(&mut self) {
        println!("Receive network messages");
    }

    pub fn outbound_network(&mut self) {
        for client in &self.clients {
            // Calculate visible entities
            // Calculate updates
            // build message
            // send message
        }

        println!("Send network messages");
    }

    pub fn tick(&mut self) {
        println!("Server tick.");
    }
}

pub struct Client {
    address: Address,
    max_packet_bytes: usize,
    outbound_tick_rate: u32,
}

pub struct Address {}



#[cfg(test)]
mod tests {

    macro_rules! count_items{
        ($name:ident) => {1};
        ($first:ident, $($rest:ident),*) => {
            1 + count_items!($($rest),*)
        }
    }


    #[test]
    fn it_works() {
        const X: usize = count_items!(a);
        const Y: usize = count_items!(a, b);
        const Z: usize = count_items!(a, b, c);
        assert_eq!(1, X);
        assert_eq!(2, Y);
        assert_eq!(3, Z);
    }
}
