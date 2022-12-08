{ lib, config, ... }: {
  users.users = {
    joerg = {
      isNormalUser = true;
      extraGroups = [ "wheel" ];
      openssh.authorizedKeys.keys = [
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKbBp2dH2X3dcU1zh+xW3ZsdYROKpJd3n13ssOP092qE joerg@turingmachine"
      ];
    };
  };

  # Assign keys from all users in wheel group
  # This is only done because nixops cant be deployed from any other account
  users.extraUsers.root.openssh.authorizedKeys.keys = lib.unique (
    lib.flatten (
      builtins.map (u: u.openssh.authorizedKeys.keys)
        (
          lib.attrValues (
            lib.filterAttrs (_: u: lib.elem "wheel" u.extraGroups)
              config.users.extraUsers
          )
        )
    )
  );
}
