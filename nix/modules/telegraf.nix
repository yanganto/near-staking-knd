{ lib, ... }:
{
  options = {
    kuutamo.telegraf.url = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "url to remote monitor";
    };

    kuutamo.telegraf.username = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "username to remote monitor";
    };

    kuutamo.telegraf.password = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "password to remote monitor";
    };
  };
}
