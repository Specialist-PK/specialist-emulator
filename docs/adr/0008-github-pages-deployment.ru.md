# 0008. Деплой на GitHub Pages после merge в `main`

[English version](0008-github-pages-deployment.md)

Статус: принято (2026-11-03)

## Контекст

Web-эмулятор должен становиться доступен на GitHub Pages после merge pull request. Перед выбором конфигурации мы изучили `DrUNE/perfect-todo`: историю workflow и упавшие запуски.

Проект показал конкретные проблемы:

- запуски [41080322756](https://github.com/DrUNE/perfect-todo/actions/runs/41080322756) и [41084857434](https://github.com/DrUNE/perfect-todo/actions/runs/41084857434) упали в `configure-pages` с `Not Found`, потому что Pages не был включён с источником GitHub Actions;
- старые версии actions использовали deprecated Node 20 и потребовали обновления;
- следующим коммитом дублированную версию Node заменили на `node-version-file`;
- Vite требуется `base` из URL Pages, а deployments не должны отменять друг друга на середине.

Нашему приложению дополнительно нужны Rust и совпадающий wasm-bindgen CLI. PR не должен ничего публиковать, но production-путь Pages необходимо проверять до merge.

## Решение

Используется `.github/workflows/deploy.yml` с триггерами `pull_request`, `push` в `main` и `workflow_dispatch`.

Build использует источники версий репозитория и запускает lint, форматирование, typecheck, TypeScript/tooling-тесты, WASM-тесты, production-сборку с base Pages и обязательный браузерный E2E. `actions/configure-pages@v6` возвращает `base_path`, который Vite получает через `BASE_PATH`. `enablement` разрешён только вне pull request: push может создать отсутствующий Pages-сайт, а недоверенный PR не меняет настройки репозитория.

Deploy выполняется только вне pull request и публикует `apps/web/dist` через `actions/deploy-pages@v5`. Permissions выдаются по jobs: build получает доступ к настройке Pages, deploy — `pages: write` и `id-token: write`. Группа `pages` ставит запуски в очередь с `cancel-in-progress: false`.

У приложения одна страница без маршрутизатора, поэтому SPA fallback `404.html` не нужен. Текущие настройки Pages уже используют `build_type: workflow`; автоматическое включение сохранено для переносимости.

## Альтернативы

- **Триггер только на `main`.** Проще и используется в `perfect-todo`, но не проверяет base Pages до merge.
- **Write permissions на весь workflow.** Меньше строк, но даёт лишние полномочия unrelated-шагам и PR.
- **Отменять старые deployments.** Меньше ожидание, но публикация может остаться в промежуточном состоянии.
- **Захардкодить `/specialist-emulator/`.** Работает для одного URL, но не для forks и другого base Pages.
- **Скопировать Node/npm workflow без изменений.** Не подходит для bun и Rust/WASM этого репозитория.

## Последствия

- (+) Каждый merge в `main` публикует проверенный эмулятор без ручного deployment.
- (+) Найденные в `perfect-todo` проблемы имеют явные защиты и consistency-тесты.
- (+) PR проверяет реальный подпуть, но не может опубликовать сайт или включить Pages.
- (−) PR дополнительно запускает web-проверки рядом с основным CI.
- (−) Настройка Pages и первый production deployment зависят от GitHub и настроек репозитория.
