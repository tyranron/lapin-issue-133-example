Reproduction of [sozu-proxy/lapin#133]
======================================

Example repo which reproduces [sozu-proxy/lapin#133].




## Reproduce steps

1. `docker-compose up` and wait RabbitMQ is ready ([RabbitMQ Management]).

2. `cargo run --bin produce` 2-3 deliveries and stop.

3. `cargo run --bin consume` all the deliveries are processed.

4. `cargo run --bin produce` again.

5. Only first delivery is processed and consuming stucks.





[sozu-proxy/lapin#133]: https://github.com/sozu-proxy/lapin/issues/133

[RabbitMQ Management]: http://127.0.0.1:15672/
