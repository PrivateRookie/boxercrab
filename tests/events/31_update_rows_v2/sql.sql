DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    `id` INT UNSIGNED AUTO_INCREMENT,
    `varchar_l` VARCHAR(100) NOT NULL,
    `varchar_s` VARCHAR(40) NOT NULL,
    `text_s` TEXT NOT NULL,
    `text_m` MEDIUMTEXT NOT NULL,
    `text_l` LONGTEXT NOT NULL,
    PRIMARY KEY (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `boxercrab` (`varchar_l`, `varchar_s`, `text_s`, `text_m`, `text_l`) VALUES ('abc', 'abc', 'abc', 'abc', 'abc');

reset master;

UPDATE boxercrab set varchar_l='xd', varchar_s='xd', text_s='xd', text_m='xd', text_l='xd';

flush logs;
