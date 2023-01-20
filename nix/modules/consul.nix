{
  # Single node consul server. Just needed for kuutamo here
  services.consul = {
    interface.bind = null;
    extraConfig = {
      server = true;
      bootstrap_expect = 1;
      bind_addr = "127.0.0.1";
    };
  };
}
