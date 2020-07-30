DROP TABLE IF EXISTS `boxercrab`;

CREATE TABLE `boxercrab` (
    `id` INT UNSIGNED AUTO_INCREMENT,
    `str` VARCHAR(40) NOT NULL,
    `int` INT NOT NULL,
    `dec` DECIMAL(10, 4) NOT NULL,
    PRIMARY KEY (`id`)
)ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

SET @val_s := "test blog";
SET @val_i := 100;
SET @val_d := 1.00;

INSERT INTO `boxercrab` (`str`, `int`, `dec`) VALUES (@val_s, @val_i, @val_d);
