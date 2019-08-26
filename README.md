Морской бой (Sea Battle)
========================

Темой соревнования по программированию искуственного интеллекта в ЛОЛ-2019 является игра в Морской бой.


Правила игры
------------

В игре участвует два игрока. Каждый игрок имеет секретное поле 10 на 10 клеток, на котором игрок расставляет свои корабли.

Всего во флотилии 10 единиц. Корабли отличаются размерами.

На поле нужно расположить единицы:

* из одной клетки - 4;
* из двух клеток – 3;
* из трех клеток – 2;
* из четырех клеток - 1.

От расстановки единиц во многом зависят шансы на победу. В правилах игры в «Морской бой» только два запрета по поводу размещения флота. Корабли не соприкасаются, клетки не располагаются по диагонали.

После того как корабли расставленны, игроки совершают "выстрелы", сообщая координаты (например, "1 1"). Промах (`miss`) передает право хода другому игроку. Попадание (`hit`) позволяет продолжать выстрелы до нерезультативного хода, а если боевая единица выбита, игрок получит соответствующее сообщение (`sunk`).

Игра прекращается если один или оба игрока нарушили правила:

* Расстановка кораблей на поле некорректна (не хватает или лишние корабли, корабли соприкасаются, и тд);
* Выстрел совершается вне поля;
* Игрок отправляет неизвестные команды вместо своего секретного поля и выстрелов.


Протокол взаимодействия
-----------------------

Для обеспечения честной игры, в игру между программами вводится судья (тоже программа). В начале игры, игроки должны составить и сообщить судье свою секретную карту -- вывести в стандартный поток вывода 10 строк по 10 символов, где `_` (подчёркивание) значит пустая клетка, а `#` (хештег) -- палуба корабля. Например:

```
####______
__________
###_###___
__________
##_##_##__
__________
#_#_#_#___
__________
__________
__________
```

После этого, можно начинать игру и делать выстрел. Для совершения выстрела, просто выведите в стандартный поток вывода координаты в диапазоне от 1 до 10, например, `1 1` или `10 10`. В ответ судья в ваш стандартный поток ввода сообщит результат: `miss` (промахнулся), `hit` (ранил), `sunk` (потопил). Судья будет вести игру до того момента как все корабли одного из игроков будут потоплены или будет выявлено нарушение правил игры.


Судья (Judge)
-------------

### Сборка

Для сборки программы судьи вам понадобятся [инструменты для сборки проектов на языке программирования Rust (nightly 2019-08-15)](https://rustup.rs/). После установки инструментов следуя инструкции, скачайте этот проект и выполните следующие команды из командной строки, находясь в корне скачанного проекта:

```
rustup toolchain add nightly-2019-08-15
cargo +nightly-2019-08-15 build --release
```

В директории `./target/release/` появится файл `judge` (или `judge.exe` на Windows).


### Запуск

Для запуска игры, судье нужно сообщить два пути к программам-ботам, например:

```
./target/release/judge.exe ./bot1.exe ./bot2.exe
```


Бонусные задания
----------------

### Игра по сети (клиент)

Протокол взаимодействия реализован на [WAMP-proto](https://wamp-proto.org/) и имеет следующий интерфейс (функции):

* `new-game` - начать новую игру.

  Параметры:

  * `player-id` (строка) - ваш уникальный ID (придумайте сами), который будет связывать все ваши игры в один рейтинг
  * `board` (массив строк) - игровое поле (10х10 - массив из 10 строк по 10 символов)

  Возвращаемое значение: `game-id` (число)
* `shoot` - совершить выстрел.

  Параметры:

  * `game-id` (число)
  * `x` (число от 1 до 10) - координата по горизонтали
  * `y` (число от 1 до 10) - координата по вертикали

  Возвращаемое значение: `status` (строка) - результат выстрела может принимать одно из четырёх значений: `miss` (промах), `hit` (попадание), `sunk` (потопление), `game-over` (конец игры).

Дополнительный интерфейс сервера включает:

* `get-games` - получить список всех игр (`game-id`)
* `get-game-info` - получить статус игры.

  Параметры:

  * `game-id`

  Возвращаемое значение: массив из пяти элементов, где первый элемент - статус игры (`waiting`, `in-progress`, `player1-win`, `player2-win`), второй - `player1-id`, третий - `player2-id`, четвёртый - поле первого игрока, пятый - поле третьего игрока.
* `get-game-log` - получить лог завершённой игры.

   Параметры:

   * `game-id`

   Возвращаемое значение: строка (см. "Визуализация сыгранной партии")

### Игра по сети (сервер)

Необходимо реализовать сервер, совместимый с протоколом, описанным для клиента.

### Визуализации сыгранной партии

Интерфейс может быть реализован как в текстовом (консольном) виде, так и в графическом (для настольных и мобильных ОС) или Web.

Получить лог игры можно будет из файла или по сети у сервера.

Файл лога выглядит следующим образом:

1. Поле игрока №1 (10 строк по 10 символов, где `_` (подчёркивание) - это пустая клетка, а `#` (хештег) - это корабль)
2. Пустая строка
3. Поле игрока №2 (10 строк по 10 символов)
4. Пустая строка
5. Выстрелы (каждый с новой строки), где выстрел - это номер игрока (1 или 2), координаты выстрела (два числа в диапазоне от 1 до 10, разделённых пробелом), результат выстрела (`miss` / `hit` / `sunk`)

Пример лога:

```
_####_###_
__________
#________#
#_____#__#
_________#
__________
___#_____#
_________#
#_________
___#___##_

#_____##__
#________#
_________#
_#_#______
_____#___#
_________#
#_####___#
#_________
#_______#_
__________

1 0 0 hit
1 1 0 miss
2 7 0 hit
2 6 0 hit
2 8 0 sunk
2 9 4 hit
2 9 5 miss
1 2 0 miss
2 9 3 hit
2 9 2 sunk
2 4 2 miss
1 3 0 miss
2 1 9 miss
1 4 0 miss
2 3 3 miss
1 5 0 miss
2 0 8 sunk
2 2 2 miss
1 6 0 hit
1 7 0 sunk
1 8 0 miss
2 3 1 miss
1 9 0 miss
2 2 0 hit
2 3 0 hit
2 1 0 hit
2 0 0 miss
1 0 1 sunk
1 1 1 miss
2 4 0 sunk
2 7 3 miss
1 2 1 miss
2 6 2 miss
1 3 1 miss
2 5 3 miss
1 4 1 miss
2 6 4 miss
1 5 1 miss
2 7 5 miss
1 6 1 miss
2 4 4 miss
1 7 1 miss
2 5 5 miss
1 8 1 miss
2 0 2 hit
2 1 2 miss
1 9 1 hit
1 0 2 miss
2 0 3 sunk
2 6 6 miss
1 1 2 miss
2 2 4 miss
1 2 2 miss
2 3 5 miss
1 3 2 miss
2 4 6 miss
1 4 2 miss
2 2 6 miss
1 5 2 miss
2 1 5 miss
1 6 2 miss
2 0 6 miss
1 7 2 miss
2 3 7 miss
1 8 2 miss
2 2 8 miss
1 9 2 sunk
1 0 3 miss
2 3 9 sunk
2 5 7 miss
1 1 3 sunk
1 2 3 miss
2 5 9 miss
1 3 3 sunk
1 4 3 miss
2 6 8 miss
1 5 3 miss
2 7 9 hit
2 8 9 sunk
2 7 7 miss
1 6 3 miss
2 8 6 miss
1 7 3 miss
2 9 7 hit
2 8 7 miss
1 8 3 miss
2 9 6 sunk
2 7 4 miss
1 9 3 miss
2 2 3 miss
1 0 4 miss
2 3 2 miss
1 1 4 miss
2 7 2 miss
1 2 4 miss
2 0 5 miss
1 3 4 miss
2 6 3 sunk
2 4 3 miss
1 4 4 miss
2 2 7 miss
1 5 4 sunk
1 6 4 miss
2 1 6 miss
1 7 4 miss
2 2 5 miss
1 8 4 miss
2 3 4 miss
1 9 4 hit
1 0 5 miss
2 5 6 miss
1 1 5 miss
2 4 7 miss
1 2 5 miss
2 6 5 miss
1 3 5 miss
2 5 8 miss
1 4 5 miss
2 3 6 sunk
```

### Интерфейс для игры человека

Интерфейс может быть реализован как в текстовом (консольном) виде, так и в графическом (для настольных и мобильных ОС) или Web.

Функции интерфейса:

* Расстановка кораблей
* Игра с противником по сети, показывая своё поле и поле противника

### Обнаружение уязвимости судьи

Уязвимость должна позволять вмешаться в игровой процесс (нарушить правила игры). ^_^
