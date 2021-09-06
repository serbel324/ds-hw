# Гарантии доставки

В данной задаче вам предстоит реализовать различные гарантии доставки сообщений в распределенной системе.

Пусть в нашей системе есть два узла и нам требуется организовать одностороннюю передачу информации между ними. Узел 
_sender_ будет принимать сообщения с информацией от своего локального пользователя и отправлять их по сети узлу 
_receiver_. Узел receiver будет принимать сообщения от sender-а и доставлять информацию своему пользователю. Под 
доставкой информации подразумевается отправка локального сообщения, идентичного исходному сообщению от пользователя 
sender-a.

Передаваемая информация может быть разного типа, в зависимости от которого требуются разные гарантии её доставки:

1. `INFO-1` - не более одного раза (at most once). Вся принятая информация должна доставляться пользователю один раз. 
   То есть доставка всей информации от sender-а не гарантируется, но доставленная информация не должна повторяться.
2. `INFO-2` - не менее одного раза (at least once). Вся информация от sender-а должна быть доставлена пользователю, 
   но при этом допускаются её повторы.
3. `INFO-3` - ровно один раз (exactly once). Вся информация от sender-а должна быть доставлена пользователю, повторы не 
   допускаются.
4. `INFO-4` - ровно один раз и с сохранением порядка. Вся информация от sender-а должна быть доставлена пользователю 
   ровно один раз и в порядке её отправки пользователем sender-a. 

Узлы могут взаимодействовать друг с другом путем обмена сообщениями через сетевой транспорт со следующими 
характеристиками: доставка сообщений не гарантируется, доставленные сообщения не искажаются, сообщения могут 
дублироваться, сохранение порядка при приеме сообщений не гарантируется, все получаемые сообщения были кем-то отправлены 
(нет сообщений "из воздуха"). Также будем предполагать, что каждое сообщение рано или поздно достигнет узла-получателя, 
если узел-отправитель будет повторять попытки отправить сообщение. В тестах это предположение соблюдается. Отказы узлов 
в данной задаче отсутствуют.

Независимо от уровня гарантий, в случае если сеть ведет себя идеально (нет потерь, дублирования и переупорядочивания 
сообщений) вся информация должна доставляться получателю в полном объеме. Это исключает, например, тривиальную 
реализацию пункта 1, которая не отправляет ничего по сети.

Несложно заметить, что реализация пункта 4 покрывает пункты 1-3. Тем не менее мы просим вас реализовать пункты 1-3 
отдельно с минимальным оверхедом для каждого из пунктов. Так вы наглядно увидите, что чем сильнее гарантия, тем больше 
дополнительных ресурсов требуется для её поддержки. На практике, в зависимости от ситуации, не всегда требуются самые 
сильные гарантии, например может быть неважен порядок доставки и "платить" за него расточительно. Поэтому если вы 
реализуете только пункт 4, то мы не зачтём вам пункты 1-3.

## Реализация

Для реализации и тестирования решения используется фреймворк [dslib](../../dslib). Перед выполнением задания 
познакомьтесь с описанием фреймворка. Также знакомству с ним посвящен первый семинар, изучите его материалы.

В папке задания размещена заготовка для решения [solution.py](solution.py). Вам надо доработать реализации классов 
`Sender` и `Receiver` так, чтобы они проходили все тесты.

### Sender

Информация для доставки передается с помощью локальных сообщений, см. метод `on_local_message()`. Все сообщения имеют 
одинаковую структуру - в единственном поле `info` содержится строка с информацией. Вы должны реализовать доставку 
информации с гарантиями, определяемыми типом сообщения. Для взаимодействия с receiver-ом вы можете использовать 
сообщения произвольного типа и структуры. Приходящие от receiver-а сообщения следует обрабатывать в методе 
`on_message()`. Также вы можете устанавливать таймеры в любом из методов и обрабатывать их наступление в `on_timer()`. 

### Receiver

Данный узел не принимает локальные сообщения, поэтому метод `on_local_message()` не используется. Сетевые сообщения 
следует обрабатывать в методе `on_message()`. Также вы можете устанавливать таймеры в любом из методов и обрабатывать 
их наступление в `on_timer()`.

Важно правильно реализовать доставку информации локальному пользователю, иначе тесты не будут проходить. Для этого вы 
должны отправить локальное сообщение с помощью метода `ctx.send_local()`. Сообщение должно быть полностью идентично 
исходному сообщению, принятому sender-ом от его пользователя, то есть иметь тот же тип и поле `info` с тем же значением. 
Других полей быть не должно.

## Тестирование

### Нативное с установленным Rust и библиотеками python

Тесты находятся в папке `test`. Для запуска тестов перейдите в эту папку и выполните команду: `cargo run`. Вывод тестов 
содержит трассы - последовательности событий во время выполнения каждого из тестов, а также финальную сводку.

### Docker

Если по какой-то причине у вас не работает нативное тестирование через cargo, предлагается воспользоваться подготовленным docker образом (в нём же тесты запускаются в GitLab CI).

Есть базовый образ `registry.gitlab.com/nanobjorn/distsys-homework`. Если у вас процессор Intel или AMD, то вам подойдет `latest`:
```
$ docker pull registry.gitlab.com/nanobjorn/distsys-homework
```

Если у вас M1 или другой arm64, то надо сделать:
```
$ docker pull registry.gitlab.com/nanobjorn/distsys-homework:arm64
$ docker image tag registry.gitlab.com/nanobjorn/distsys-homework:arm64 registry.gitlab.com/nanobjorn/distsys-homework:latest
```

Теперь соберём образ c вашим решением и тестами:

```
docker build .. -t guarantees -f Dockerfile
```

И запустим тесты:

```
docker run --rm guarantees
```

Если вы поменяли solution.py, то можно запустить тесты без пересборки образа:

```
docker run --rm -v `pwd`:/guarantees/ guarantees
```

Опция -v смонтирует вашу папку задания внутрь контейнера, соответственно запуск `cargo run` будет использовать и создавать локальные файлы, а не внутри контейнера.

## Оценивание

Распределение баллов по требованиям и тестам:

- Поддержка `INFO-1` - 2 балла
  - проходят все тесты с данным префиксом
  - минимальный оверхед для данной гарантии
- Поддержка `INFO-2` - 2 балла
  - проходят все тесты с данным префиксом
  - минимальный оверхед для данной гарантии
- Поддержка `INFO-3` - 2 балла
  - проходят все тесты с данным префиксом
  - минимальный оверхед для данной гарантии
- Поддержка `INFO-4` - 4 балла
  - проходят все тесты с данным префиксом

## Сдача

Описано в корневом [README.md](../README.md).