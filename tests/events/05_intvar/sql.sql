DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    `id` INT UNSIGNED AUTO_INCREMENT,
    `title` VARCHAR(100) NOT NULL,
    `author` VARCHAR(40) NOT NULL,
    `time` DATETIME NOT NULL,
    `score` INT DEFAULT 0,
    PRIMARY KEY (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `boxercrab` (`title`, `author`, `time`, `score`) VALUES ('test blog', 'xd', '2020-07-11', 12);
INSERT INTO `boxercrab` (`title`, `author`, `time`, `score`) VALUES ('test blog', 'xd', '2020-07-11', LAST_INSERT_ID());