iron-test
=========

A ``request_get()`` and ``request_post()`` function to properly and easily test Iron handlers without firing up a local HTTP server.

Invoke the functions by passing an Iron chain (``iron::middleware::Chain``) and - in the case of a POST request - a ``&str`` containing the body payload.

```
	#[test]
	fn test_hello_get()
	{
		let iron = IronServer::new().create_iron();
		let response = request_get(&iron, "/hello").unwrap();

		assert!(response.status.unwrap().is_client_error());
	}
```