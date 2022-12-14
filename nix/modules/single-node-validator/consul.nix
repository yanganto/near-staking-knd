{
  # Single node consul server. Just needed for kuutamo here
  services.consul = {
    interface.bind = "lo";
    extraConfig = {
      server = true;
      bootstrap_expect = 1;
    };
  };
}
