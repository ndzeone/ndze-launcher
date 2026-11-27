# Contributing to NDZE / Участие в разработке

Contributions in English or Russian are welcome: bug reports, documentation, translations, tests and focused fixes.

## Workflow

1. Check [existing issues](https://github.com/ndzeone/ndze-launcher/issues) and discuss significant changes before implementing them.
2. Fork the repository, clone your fork and create a topic branch from the default branch.
3. Make a small, focused change. Follow the surrounding code style.
4. Test on the relevant platform and document any checks you could not run.
5. Open a pull request describing the problem, solution, tests and potential risks. Include screenshots for UI changes when appropriate.

## Development

- Windows: Rust, Node.js 22+, Visual Studio C++ Build Tools, WebView2; run `npm ci` and `npm run tauri dev` in `windows-launcher/`. Run relevant Rust formatting, lint and test commands where supported. Test system tweaks on a suitable Windows test machine or VM and verify rollback.
- iOS: macOS, Xcode and XcodeGen; generate the project using `xcodegen generate` in `ios-app/` and run the relevant Xcode tests.
- Website: use the scripts declared in `website/package.json` and verify the production build.

See [README.md](README.md) for detailed setup and platform limitations.

Never commit credentials, private logs, signing files or personal data. Do not introduce hidden telemetry or unnecessary elevated privileges. Review AI-generated code before submitting it. Report vulnerabilities according to [SECURITY.md](SECURITY.md).

By submitting a contribution, you confirm that you have the right to submit it under the repository's [MIT license](LICENSE). Third-party materials remain subject to their own licenses.

## На русском

Проверьте существующие задачи, создайте форк и отдельную ветку, внесите небольшое изменение и отправьте Pull Request с описанием и результатами проверок. Если тестирование невозможно, укажите это. Не публикуйте ключи и персональные данные. Вопросы безопасности направляйте по правилам SECURITY.md. Принимаются документация, переводы, тесты, исправления и отзывы пользователей.
