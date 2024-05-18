# Swagger to TypeScript/JavaScript Service Generator

Bu proje, bir Swagger JSON dosyasını okuyarak TypeScript ve JavaScript dosyaları oluşturan bir araçtır. Oluşturulan dosyalar, Swagger tanımlarına dayalı olarak TypeScript arayüzleri ve HTTP servis fonksiyonları içerir.

## Kurulum ve Kullanım

### Gereksinimler

- [Rust](https://www.rust-lang.org/) (Bu projeyi derlemek ve çalıştırmak için Rust dilini ve paket yöneticisini kurmanız gerekmektedir)

### Adımlar

1. Bu projeyi klonlayın veya indirin.
2. `swagger.json` dosyasını proje dizinine yerleştirin.
3. Projeyi derleyin ve çalıştırın.

```sh
git clone https://github.com/muhtalipdede/swagger-generator.git
cd swagger-generator
cargo build
cargo run

# veya

cargo run --release
```

4. Oluşturulan TypeScript ve JavaScript dosyalarını `output` dizininde bulabilirsiniz.

## Katkıda Bulunma

Bu proje, her türlü katkıya açıktır. Lütfen bir sorun bildirin veya bir istek gönderin.

## Lisans

Bu proje MIT lisansı altında lisanslanmıştır. Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakın.
