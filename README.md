### EX_AUCTION
### Реализация упрощенного аукциона на основе блокчейн-фреймворка Exonum

#### Сборка и запуск
Проект выполнен в виде единой ноды, конфигурация сети не требуется
```console
qshell@localhost:~$ cargo build --release && cargo run --release
qshell@localhost:~$ Running `target/release/ex_auction`
Blockchain is ready for transactions!
```


#### Интеграционное тестирование
Выполняются тесты по созданию лотов, созданию заявок на лоты, получению заявок по лоту и закрытию аукциона 
```console
qshell@localhost:~$ cargo test --test api -- --nocapture
```
