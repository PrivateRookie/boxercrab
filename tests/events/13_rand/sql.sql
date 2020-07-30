DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    i INT AUTO_INCREMENT PRIMARY KEY,
    c VARCHAR(10)
);

INSERT INTO `boxercrab` (i, c) VALUES(FLOOR(RAND() * 100), 'abc');

flush logs;
