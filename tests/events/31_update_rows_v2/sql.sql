DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    `id` INT UNSIGNED AUTO_INCREMENT,
    `varchar_l` VARCHAR(100) NOT NULL,
    `varchar_s` VARCHAR(40) NOT NULL,
    `text_s` TEXT NOT NULL,
    `text_m` MEDIUMTEXT NOT NULL,
    `text_l` LONGTEXT NOT NULL,
    `num_float` FLOAT NOT NULL,
    `num_double` DOUBLE NOT NULL,
    `num_decimal` DECIMAL(10, 4),
    PRIMARY KEY (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `boxercrab` (`varchar_l`, `varchar_s`, `text_s`, `text_m`, `text_l`, `num_float`, `num_double`, `num_decimal`) VALUES ('abc', 'abc', 'abc', 'abc', 'abc', 1.0, 2.0, 3.0);

reset master;

UPDATE boxercrab set varchar_l='xd', varchar_s='xd', text_s='xd', text_m='xd', text_l='xd', num_float=4.0, num_double=4.0, num_decimal=4.0;

flush logs;
