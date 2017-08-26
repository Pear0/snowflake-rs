use std::io;
use std::sync::{Arc, Mutex};
use bytes::{BytesMut, BufMut, BigEndian};

use futures::{Async, future, Future, Poll};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;

use generator::IDGenerator;

type BoxFuture<T, E> = ::std::boxed::Box<Future<Item = T, Error = E> + Send>;

pub struct IDCodec;

pub struct IDRequest {
    request_code: u8,
}

impl Decoder for IDCodec {
    type Item = IDRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() > 0 {
            Ok(Some(IDRequest { request_code: src.split_to(1).first().unwrap().clone() }))
        } else {
            Ok(None)
        }
    }
}

pub struct IDResponse {
    id: i64,
}

impl Encoder for IDCodec {
    type Item = IDResponse;
    type Error = io::Error;

    fn encode(&mut self, item: IDResponse, dst: &mut BytesMut) -> Result<(), io::Error> {
        dst.put_i64::<BigEndian>(item.id);

        Ok(())
    }
}

pub struct IDProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for IDProto {
    /// For this protocol style, `Request` matches the `Item` type of the codec's `Decoder`
    type Request = IDRequest;

    /// For this protocol style, `Response` matches the `Item` type of the codec's `Encoder`
    type Response = IDResponse;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, IDCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(IDCodec))
    }
}

pub struct IDFuture {
    generator: Arc<Mutex<IDGenerator + Send + Sync>>,
}

impl Future for IDFuture {
    type Item = i64;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut generator = self.generator.lock().unwrap();

        match generator.generate() {
            Some(id) => Ok(Async::Ready(id)),
            None => Ok(Async::NotReady),
        }
    }
}

pub struct IDService {
    generator: Arc<Mutex<IDGenerator + Send + Sync>>,
}

impl Service for IDService {
    // These types must match the corresponding protocol types:
    type Request = IDRequest;
    type Response = IDResponse;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.

        if req.request_code != 0x50 {
            return Box::new(future::ok(IDResponse { id: -1 }))
        }

        Box::new(IDFuture { generator: self.generator.clone() }.map(|id| IDResponse { id: id }))
    }
}


pub fn start_server<E: IDGenerator + Send + Sync + Sized + 'static>(generator: E, bind_addr: &str) {
    // Specify the localhost address
    let addr = bind_addr.parse();

    if let Err(_) = addr {
        error!("Failed to resolve: {:?}", bind_addr);
        return
    }

    info!("Starting server on {}", bind_addr);

    // The builder requires a protocol and an address
    let server = TcpServer::new(IDProto, addr.unwrap());

    let generator = Arc::new(Mutex::new(generator));

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(move || Ok(IDService { generator: generator.clone() }));
}

