DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    i INT AUTO_INCREMENT PRIMARY KEY,
    c VARCHAR(10)
);

INSERT INTO `boxercrab` (i, c) VALUES(LAST_INSERT_ID()+1, 'abc');

flush logs;
