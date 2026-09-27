# Grimorio de los Mil Nombres

[English](README.md) · [Español](README.es.md) · [Русский](README.ru.md)

<p align="center">
  <img src="grimoriodelosmilnombres.png" alt="Grimorio de los Mil Nombres — banner" />
</p>

<p align="center"><em>Где шрамы учатся говорить, а имена отказываются заикаться дважды.</em></p>

<p align="center"><strong>Три языка. Два инструмента. Один голос для того, что выжило.</strong></p>

---

**Grimorio de los Mil Nombres** — не список слов. Это **палата шёпотов**: книга, что отвечает, когда живое дело просит строку знания или истинное имя, выкованное из осей шрама, резонанса, тени и мифа. Написанный на Rust (с отдельным Python-инструментом русских имён), он хранит то, что прежде жило как модуль `whisper` в Espiralismo: таблицы локалей, грамматика красоты и ужаса, обломки мудрости и кузница поколенческих эпитетов.

Он не владеет спиралью. Он **отвечает**. Хост строит упорядоченный uplink, вызывает `answer` и читает упорядоченный downlink — по-английски, по-испански или по-русски; у каждого языка свой отказ уронить проклятие на ствол, который его не выдержит.

> **Живой портрет входит. Имя с историей выходит.** Передайте книге важные черты — она подберёт слова, чей смысл и грамматика способны их вынести.

## Откройте книгу за минуту

Rust-crate и CLI находятся в этом репозитории. Установите Rust и попросите первые эпитеты:

```bash
cargo run -p grimorio -- epithet --count 3 --seed 424242 --spanish
```

Или попросите голос мудрости ответить фрагментом, окрашенным эхом:

```bash
cargo run -p grimorio -- wisdom --mix 42 --scar-mass 10 --lang ru
```

Тот же движок можно подключить к приложению как библиотеку:

```rust
use grimorio::{answer, Language, NarrativeEcho, Uplink};

fn main() {
    let petition = Uplink::wisdom(Language::Russian, 42, NarrativeEcho::default());
    let reply = answer(&petition);
    println!("{}", reply.text);
}
```

Запустите `cargo run -p grimorio -- help`, чтобы увидеть все команды CLI, или сначала прочитайте [контракт для хостов](CONTRACT.md).

## Две палаты под одним именем

Гримуар держит **две разные поверхности**. Путать их — значит делать имена пустыми.

**Шёпоты / эпитеты** (`crates/grimorio`) — живая книга. Голоса мудрости и поколенческих эпитетов, встроенные TOML локалей, кузница, что превращает портрет standout в имя. Именно от этого зависят хосты.

**Русские собственные имена** (`tools/ru-names`) — отдельный генератор. Мифологические леммы с литературным склонением (`name` / `patron` / `verbal`), что правят `[epithet].proper_names` внутри собственного `ru_vocab.toml` Гримуара. Он питает книгу; он не есть книга.

Ухаживайте за обоими, если хотите, чтобы русские эпитеты держали честную грамматику. Зовите Rust-crate, когда нужен шёпот или имя *сейчас*.

## Что делает книга (языком работы)

Открывая Гримуар, вы посылаете петицию с **языком** и **голосом**.

**Мудрость** отвечает одной обрывочной строкой — частичное знание, почти понятое — смещённой нарративным эхом: шрамы решётки, доминирующий тон, редкие события, настройка души. Это не объяснение. Это фраза, на которой завеса почти закрылась.

**Поколенческие эпитеты** — истинные имена для того, кто возобладал. Из портрета standout — label, fitness, viability, resonance, mutation, memory, shadow — кузница собирает основы, определители, патронимы, титулы и глагольные состояния, чтобы ни один эпитет не заикался о бездне дважды, и ни одно проклятие не легло туда, где грамматика не удержит.

**Образцовые эпитеты** — книга, практикующая в одиночку: синтетические портреты для CLI и детерминированные последовательности, когда вы просто хотите слышать, как поёт кузница.

Хосты вроде **Espiralismo** реэкспортируют crate через тонкий адаптер, отображают живые сущности в `StandoutPortrait` и потребляют текст downlink. Гримуар никогда не касается эволюции; он только именует и шепчет.

---

## Та же работа, простыми сигилами (техническая карта)

| Путь | Назначение |
|------|------------|
| `crates/grimorio` | Библиотека Rust и CLI `grimorio`. |
| `crates/grimorio/src/link.rs` | `Uplink`, `UplinkVoice`, `Downlink`, `UplinkDraft`, `answer`. |
| `crates/grimorio/src/epithet.rs`, `semantic.rs`, `grammar.rs` | Кузница эпитетов, семантическая совместимость и грамматическое согласование. |
| `crates/grimorio/src/wisdom.rs`, `locale.rs`, `locales/` | Таблицы мудрости и TOML-словари `en` / `es` / `ru`. |
| `crates/grimorio/src/main.rs` | CLI: `help`, `answer`, `epithet`, `wisdom`. |
| `tools/ru-names` | Python-инструмент `grimorio-ru-names` (каталог имён). |
| `CONTRACT.md` | `grimorio.uplink/v1` · `grimorio.downlink/v1` · `grimorio.proper_names/v1`. |

**Crate:** `grimorio` (текущая версия **0.1.0**). **Имя проекта:** **Grimorio de los Mil Nombres**.

### Публичная поверхность (Rust)

`answer`, `Uplink`, `UplinkDraft`, `UplinkVoice`, `Downlink` · `Language`, `NarrativeEcho`, `StandoutPortrait` · `forge_sample`, `WhisperHub`, помощники мудрости.

### Путь хоста (Espiralismo)

```toml
grimorio = { path = "../Grimorio de los Mil Nombres/crates/grimorio" }
```

Адаптер: `src/whisper.rs` — реэкспорт + `standout_epithet_for_report` / построители uplink.

---

## Как пройти круг

### Rust CLI (шёпоты и эпитеты)

```bash
cd "/path/to/Grimorio de los Mil Nombres"
cargo test -p grimorio
cargo run -p grimorio -- help
cargo run -p grimorio -- epithet
cargo run -p grimorio -- epithet --count 10 --seed 424242 --spanish
cargo run -p grimorio -- answer --label keeper --shadow 0.9 --lang ru
cargo run -p grimorio -- wisdom --mix 42 --scar-mass 10
cargo run -p grimorio -- answer --help
```

Опциональная установка:

```bash
cargo install --path crates/grimorio
grimorio epithet --count 5
```

Флаги uplink **необязательны**. `UplinkDraft` дополняет пропущенное. Намёки standout (напр. `--label`, `--shadow`) выбирают `generation_epithet`, если голос не задан; намёки wisdom — `wisdom`; иначе `sample_epithet`.

### Python-инструмент (русские собственные имена)

```bash
cd tools/ru-names
python3 -m pip install -e ".[dev]"
grimorio-ru-names list
grimorio-ru-names apply --target ../../crates/grimorio/src/locales/ru_vocab.toml
```

Из Espiralismo: `bash scripts/sync-grimorio-names.sh`.

Порядок полей: [CONTRACT.md](CONTRACT.md) · [tools/ru-names/README.md](tools/ru-names/README.md).

---

## Лицензия

Двойное лицензирование:

- **AGPL-3.0** — см. [LICENSE](LICENSE)
- **Коммерческая** — для проприетарного использования; см. [COMMERCIAL.md](COMMERCIAL.md)

Copyright © 2026 Fidel Herrera Castro (fherrera221@gmail.com).

## Лицензия тона

Этот README говорит сначала метафорой, затем **таблицами и списками**, чтобы люди и духи кода удержали и *намерение*, и *интерфейс*: упорядоченный uplink, обозримый downlink, мост между **шрамом**, **языком** и **истинным именем**. Спираль может спросить; Гримуар отвечает, не владея дыханием.

*Выкованное имя не должно заикаться о бездне дважды.*
