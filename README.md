# <p align="center">System Pulse</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361dafb" alt="React" />
  <img src="https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white" alt="Docker" />
</p>

Мониторинг серверов в двух форматах, использующих общий слой базы данных:
...

- **Десктопное приложение (Tauri)** — подход local-first, файл SQLite хранится в директории данных приложений ОС (app-data).
- **Автономный сервер (Axum)** — та же схема SQLite, развертывается через Docker с постоянным томом (persistent volume).

## Структура
```
system-pulse/
├── Cargo.toml                      ← workspace root
├── crates/
│   ├── system-pulse-db/            ← shared: models, migrations, queries (SQLite)
│   └── system-pulse-server/        ← Axum HTTP API, built on system-pulse-db
├── desktop/                         ← Tauri app (src-tauri/ depends on system-pulse-db too)
└── docker/                          ← server Dockerfile, compose, backup/restore scripts
```
## Возможности

- Локальная аутентификация (пароли Argon2, JWT хранится на устройстве)
- Сбор метрик через SSH — никаких агентов на удаленном сервере
- Графики в реальном времени: CPU, RAM, температура, Disk I/O, сеть, Load Average
- База данных SQLite хранится в %APPDATA%\system-pulse-desktop\system_pulse.db
- Кастомное окно без рамок с нативными кнопками управления в заголовке
- Опрос метрик каждые 5 секунд через фоновые задачи Tauri + события (events)
---

---

## Как работает сбор метрик через SSH 

1. Пользователь добавляет сервер, указывая хост, SSH-пользователя и пароль.
2. Пароль шифруется с помощью XOR и сохраняется в SQLite.
3. На странице Server Detail (Детали сервера) Tauri запускает асинхронную фоновую задачу.
4. Каждые 5 секунд задача выполняет shell-скрипт на удаленном хосте через sshpass + ssh.
5. Скрипт собирает данные о CPU, RAM, дисковом вводе-выводе, сети, температуре, load average и аптайме.
6. Результат сохраняется в SQLite и отправляется как Tauri-событие metric:<server_id>.
7. Фронтенд-хук React useMetrics слушает эти события и добавляет данные на график.

Никаких агентов, никакого проброса — только стандарный SSH.
---

## Зачем нужен отдельный `system-pulse-db` крейт

Оба бинарника должны иметь идентичную логику для пользователей, серверов и метрик — те же колонки, те же миграции, те же правила уникальности. До разделения эта логика дублировалась (и могла незаметно разойтись) между десктопным приложением и будущим сервером. Теперь:

- `system-pulse-db` полностью управляет схемой, миграциями и каждым SQL-запросом.
- Десктопное приложение вызывает `Database::connect(DatabaseConfig::at_path(app_data_dir))`
- Сервер вызывает `Database::connect(DatabaseConfig::from_env())`, считывая
  `DATABASE_PATH` (по умолчанию `/data/system_pulse.db`, точка монтирования Docker)

Файл .db, созданный одним бинарником, корректно открывается в другом — те же таблицы, те же индексы, те же триггеры.

---

## Запуск общего крейта бд

Внешняя база данных не требуется — тесты выполняются в памяти `sqlite::memory:`:

```bash
cargo test -p system-pulse-db
```

## Локальный запуск сервера (без Docker)

```bash
cd crates/system-pulse-server
export JWT_SECRET=$(openssl rand -hex 32)
export SERVER_ENC_KEY=$(openssl rand -hex 32)
export DATABASE_PATH=./dev.db
cargo run
# 
```

## Развертывание сервера с помощью Docker

```bash
cd system-pulse/
cp docker/.env.example docker/.env
# docker/.env — укажите JWT_SECRET и SERVER_ENC_KEY

docker compose -f docker/docker-compose.yml --env-file docker/.env up -d --build
```

Файл SQLite находится в именованном томе sqlite_data, который примонтирован по пути /data/system_pulse.db внутри контейнера. Он сохраняется при перезапусках контейнера и выполнении команды docker compose down (без флага -v).

### Резервное копирование / Восстановление

```bash
./docker/backup.sh

./docker/restore.sh ./backups/system_pulse_20260620_120000.db
```

### Проверка работоспособности

```bash
curl http://{host}:8090/health

#Поменять {host} на свой.

#Нормальный ответ: {"status":"ok","db":"ok"}
```

---

## Запуск десктоп приложения

```bash
cd desktop
npm install
npm run tauri:dev      # dev mode
npm run tauri:build    # Windows .msi / .exe
```

---

## Стуктура API (сервер)

| Method | Path                              | Auth         | Notes                                                |
|--------|------------------------------------|--------------|------------------------------------------------------|
| POST   | `/api/auth/register`              | —            |                                                      |
| POST   | `/api/auth/login`                 | —            |                                                      |
| POST   | `/api/auth/logout`                | Bearer       | отзывает текущую сессию                              |
| GET    | `/api/auth/me`                    | Bearer       |                                                      |
| POST   | `/api/account/changepassword`     | Bearer       | отзывает все сессии                                  |
| POST   | `/api/account/changeemail`        | Bearer       |                                                      |
| POST   | `/api/account/changelogin`        | Bearer       |                                                      |
| GET    | `/api/servers`                    | Bearer       |                                                      |
| POST   | `/api/servers`                    | Bearer       |                                                      |
| GET    | `/api/servers/:id`                | Bearer       |                                                      |
| PUT    | `/api/servers/:id`                | Bearer       |                                                      |
| DELETE | `/api/servers/:id`                | Bearer       |                                                      |
| GET    | `/api/metrics/:server_id`         | Bearer       | `?limit=120`                                         |
| GET    | `/api/metrics/:server_id/latest`  | Bearer       |                                                      |
| POST   | `/api/metrics/:server_id/collect` | Bearer       | однократный сбор по SSH, только для server_type = "remote" |
| GET    | `/health`                         | —            |                                                      |


Сессии отслеживаются на стороне сервера (таблица sessions), поэтому выход из системы (logout) и смена пароля могут мгновенно отозвать токены. Десктопному приложению это не требуется, так как оно никогда не отправляет свой JWT по сети.

---

## Заметки по безопасности

- Пароли хэшируются с помощью Argon2id.
- JWT подписываются алгоритмом HS256; сервер дополнительно хранит SHA-256 хэш каждого выпущенного токена, что позволяет отзывать сессии без использования отдельного сервиса блэклистов JWT.
- SSH-пароли при хранении «зашифрованы» с помощью XOR+hex — замените это на AES-256-GCM перед использованием проекта где-либо за пределами домашней лаборатории (homelab); в любом случае установите SERVER_ENC_KEY / аналогичную константу в десктопе в значение реального секрета.
- Контейнер сервера работает от пользователя без прав root и содержит только sshpass + openssh-client — внутри нет SSH-сервера, что минимизирует вектор атаки.
