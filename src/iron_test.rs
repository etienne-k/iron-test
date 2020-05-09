use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::io::{Read, Write};
use iron::middleware::Handler;
use log::*;

pub fn request_get(iron: &iron::Iron<iron::middleware::Chain>, context: &str) ->
	Result<iron::Response, ()>
{
	return request(iron, "GET", context, "");
}

pub fn request_post(iron: &iron::Iron<iron::middleware::Chain>, context: &str, body: &str) ->
	Result<iron::Response, ()>
{
	return request(iron, "POST", context, body);
}

fn request(iron: &iron::Iron<iron::middleware::Chain>, method: &str, context: &str, body: &str) ->
	Result<iron::Response, ()>
{
	let address = SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3000));
	let mut request_stream = QueueStream::new();

	write_request(&mut request_stream, method, context, body);

	let mut buffer_reader: hyper::buffer::BufReader<&mut dyn hyper::net::NetworkStream> =
		hyper::buffer::BufReader::new(&mut request_stream);
	let hyper_request: hyper::server::Request =
		hyper::server::Request::new(&mut buffer_reader, address).unwrap();
	let mut request: iron::Request = iron::Request::from_http(
		hyper_request, address, &iron::Protocol::http()).unwrap();

	return Ok(iron.handler.handle(&mut request).unwrap());
}

fn write_request(stream: &mut QueueStream, method: &str, context: &str, body: &str)
{
	let mut request = String::new();
	request.push_str(format!("{} {} HTTP/1.1\r\n", method, context).as_str());
	request.push_str("Host: localhost\r\n");
	//request.push_str("Content-Type: application/json\r\n");
	request.push_str(format!("Content-Length: {}\r\n", body.len()).as_str());
	request.push_str("\r\n");
	request.push_str(body);
	request.push_str("\r\n");

	stream.write_all(request.as_bytes()).unwrap();
}

pub fn get_request_body(request: &mut iron::Request) -> std::io::Result<String>
{
	let mut response_body = String::new();

	return match request.body.read_to_string(&mut response_body)
	{
		Ok(_size) => Ok(response_body),
		Err(error) => {
			log::warn!("Request body could not be read: {}", error);
			return Err(error);
		}
	};
}

#[allow(dead_code)]
fn get_hyper_request_body(request: &mut hyper::server::Request) -> String
{
	let mut buffer = String::new();

	request.read_to_string(&mut buffer).unwrap();

	return buffer;
}

#[allow(dead_code)]
fn get_buffer_as_string(reader: &mut hyper::buffer::BufReader<&mut dyn hyper::net::NetworkStream>) -> String
{
	let mut buffer = String::new();

	reader.read_to_string(&mut buffer).unwrap();

	return buffer;
}

pub fn get_response_body(response: iron::Response) -> std::io::Result<String>
{
	let mut response_stream = QueueStream::new();
	let mut response_body = String::new();

	let mut body = response.body.unwrap();

	match body.write_body(&mut response_stream)
	{
		Err(error) => {
			log::warn!("Response body could not be written into the buffer: {}", error);
			return Err(error);
		},
		_ => ()
	}

	return match response_stream.read_to_string(&mut response_body)
	{
		Ok(_size) => Ok(response_body),
		Err(error) => {
			log::warn!("Response body could not be read: {}", error);
			return Err(error);
		}
	};
}

/// A simple FIFO stream into which can be written and from which can be read.
#[derive(Clone)]
struct QueueStream
{
	data: std::collections::VecDeque<u8>
}

impl QueueStream
{
	pub fn new() -> Self
	{
		return QueueStream {
			data: std::collections::VecDeque::new()
		};
	}

	fn min(a: usize, b: usize) -> usize
	{
		if a <= b
		{
			return a;
		}
		else
		{
			return b;
		}
	}
}

impl Read for QueueStream
{
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
	{
		let mut read_bytes: usize = 0;
		let max_readable = Self::min(self.data.len(), buf.len());

		for i in 0 .. max_readable
		{
			buf[i] = self.data.pop_front().unwrap();
			read_bytes += 1;
		}

		return Ok(read_bytes);
	}
}

impl Write for QueueStream
{
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
	{
		let mut written_bytes: usize = 0;
		let x = buf.len();

		for i in 0 .. x
		{
			self.data.push_back(buf[i]);
			written_bytes += 1;
		}

		return Ok(written_bytes);
	}

	fn flush(&mut self) -> std::io::Result<()>
	{
		return Ok(());
	}
}

impl hyper::net::NetworkStream for QueueStream
{
	fn peer_addr(&mut self) -> std::io::Result<SocketAddr>
	{
		log::warn!("STUB peer_addr()");
		let address = SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3000));
		return Ok(address);
	}

	fn set_read_timeout(&self, _duration: Option<std::time::Duration>) -> std::io::Result<()>
	{
		log::warn!("STUB set_read_timeout()");
		return Ok(());
	}

	fn set_write_timeout(&self, _duration: Option<std::time::Duration>) -> std::io::Result<()>
	{
		log::warn!("STUB set_write_timeout()");
		return Ok(());
	}
}