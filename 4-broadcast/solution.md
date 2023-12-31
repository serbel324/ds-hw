# Решение
## Что храним на нодах?
1. Множество seen - хеши (так я называю уникальную строку, приписываемое рассылаемому сообщению в момент первой отправки) всех когда-либо отправленных сообщений
2. Словарь messages - пары вида { хеш : MessageInfo} - информация о всех когда-либо полученных сообщениях. MessageInfo содержит текст сообщения, количество полученных ACK-ов (consensus - по умолчанию 1), флаг доставки (по умолчанию False)
3. Счетчик вызовов SEND

## Что там с сообщениями?
### BCAST
Хранит текст и хеш
### ACK
Хранит только хеш

# Что происходит?
### Нода получила локальное сообщение SEND
1. Генерируем хеш - строка вида "${номер_ноды}:${уникальное_число_внутри_ноды}", увеличиваем счетчик
2. Добавляем информацию о сообщении в seen и messages
3. Рассылаем сообщения всем кроме себя (оптимизация аж на константу!)
4. Если вдруг хитрый проверяющий решил создать кластер из одной или двух корректных нод, проверяем, что отправили сообщение сами себе
### Нода получила сообщение BCAST
1. Если видим сообщение впервые, добавляем информацию о нем в seen и messages, а потом щедро пересылаем это сообщение всем нодам (кроме себя, не забываем про константные оптимизации)
2. Потом отправляем сообщение ACK отправителю и проверяем, что отправили сообщение сами себе, если нас вдруг решат поймать на вырожденном случае из двух нод.
### Нода получила сообщение ACK
1. Увеличиваем счетчик consensus для данного сообщения. 
2. Проверяем, надо ли доставлять сообщение

## А как, собственно, мы необходимость доставки проверяем?
1. Смотрим, сколько было ACK-ов и было ли сообщение уже доставлено. Если количество ACK-ов больше или равно половине нод, а сообщение не было доставлено, то доставляем сообщение
2. Вывешиваем флаг доставки сообщения

# А почему оно работает? 
## No duplication
Потому что у нас есть явный флаг доставки.
## No Creation
Текст мы берем из структуры MessageInfo из messages, где он мог оказаться либо в момент получения SEND, либо в момент получения BCAST, когда он пересылается без измненений. 
## Validity
Корректный узел отправит всем нодам сообщение, потом получит не меньше половины ACK-ов (по условию), а потом доставит сообщение
## Uniform Agreement
Если сообщение BCAST или SEND придет хотя бы на один корректный узел, то он успешно отправит его на все корректные узлы и все они доставят сообщение по соображением из п. VALIDITY
Если сообщение не попало ни на один из корректных узлов, то ни один некорректный узел не получил достаточное количество ACK-ов, а значит не мог доставить сообщение.
## CAUSAL ORDER
Скажем так, есть ненулевая вероятность выполнения этого требования.