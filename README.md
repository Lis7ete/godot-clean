# 🦀 godot-clean

> Высокопроизводительный параллельный CLI-санитар и линтер для проектов на Godot 3 и Godot 4, написанный на Rust.

[![CI](https://github.com/Lis7ete/godot-clean/actions/workflows/ci.yml/badge.svg)](https://github.com/Lis7ete/godot-clean/actions)
[![Rust](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Godot](https://img.shields.io/badge/Supports-Godot_3_&_4-478CBF?style=flat-square&logo=godotengine&logoColor=white)](https://godotengine.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

---

## 💡 Зачем это нужно?

В процессе разработки игры на Godot файлы часто перемещаются, переименовываются или удаляются. В результате:
- В сценах `.tscn` и ресурсах `.tres` остаются **битые пути** вида `res://...` или неразрешенные `uid://...`, приводящие к ошибкам при загрузке.
- В папках накапливаются **осиротевшие файлы `.import`** от давно удаленных картинок и звуков.
- В проекте пылятся **неиспользуемые ассеты**, раздувающие размер итогового билда.

**`godot-clean`** проверяет целостность проекта и находит проблемы за доли секунды.

---

## ✨ Возможности

- ⚡ **Многопоточность на Rayon:** параллельный обход и анализ тысяч сцен, ресурсов и скриптов.
- 🔍 **Комплексный поиск связей:**
  - Полная поддержка `res://` путей.
  - Поддержка ссылок Godot 4 `uid://...` с разрешением через `.uid` файлы.
  - Поддержка относительных путей (`./bullet.tscn`, `../player.gd`).
  - Сканирование GDScript (`.gd`), C# (`.cs`), сцен (`.tscn`), ресурсов (`.tres`, `.material`) и шейдеров (`.gdshader`).
- 🧹 **Очистка сирот:** находит `.import` файлы без исходников и безопасно удаляет их (с поддержкой `--dry-run`).
- 📦 **Детект лишних ассетов:** выявляет кандидатов на удаление среди текстур, звуков и моделей с подсчетом занимаемой памяти.
- 🛡️ **Надежность и обработка I/O:** сбор всех ошибок доступа и чтения файлов, строгий режим `--strict` для CI/CD.
- 🤖 **Поддержка JSON:** интеграция с CI/CD и скриптами через `--format json`.

---

## 🚀 Установка и сборка

Требуется установленный компилятор Rust:

```bash
git clone https://github.com/Lis7ete/godot-clean.git
cd godot-clean
cargo build --release
```

Собранный бинарник: `target/release/godot-clean`.

---

## 💻 Использование

### 1. Проверка проекта (по умолчанию)
```bash
godot-clean
# или с явным указанием пути
godot-clean path/to/my-godot-project
```

### 2. Строгий режим для CI/CD
Завершается с кодом `1`, если найдены предупреждения или ошибки ввода/вывода:
```bash
godot-clean check --strict
```

### 3. Экспорт в JSON
```bash
godot-clean check --format json
```

### 4. Очистка осиротевших файлов .import
```bash
# Предварительный просмотр (без удаления)
godot-clean fix --dry-run

# Реальное удаление
godot-clean fix
```

### 5. Статистика проекта
```bash
godot-clean stats
```

---

## 🛠️ Стек
- **Rust 2024 Edition**
- `rayon` — параллельная обработка файлов
- `clap` — парсинг CLI аргументов и субкоманд
- `serde` / `serde_json` — сериализация отчетов в JSON
- `walkdir` — обход дерева проекта
- `regex` — распознавание путей и UID
- `colored` — терминальный UI

---

Автор: [Lis7ete](https://github.com/Lis7ete) • Telegram-канал: [@lis_7ete](https://t.me/lis_7ete)
