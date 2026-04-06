{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (builtins) concatStringsSep;
  inherit (lib.modules) mkIf;
  inherit (lib.meta) getExe;
  inherit (lib.options) mkEnableOption mkOption mkPackageOption;
  inherit (lib.types) int nullOr str nonEmptyListOf;
  inherit (lib.strings) optionalString;

  ttlFlag = optionalString (cfg.ttl != null) "--ttl=${toString cfg.ttl}";
  fqdnFlag = optionalString (cfg.fqdn != null) "--fqdn='${cfg.fqdn}'";
  subdomainFlags = concatStringsSep " " (map (subdomain: "--subdomain='${subdomain}'") cfg.subdomains);

  cfg = config.services.avahi-subdomains;
in {
  options.services.avahi-subdomains = {
    enable = mkEnableOption "avahi-subdomains";
    package = mkPackageOption pkgs "avahi-subdomains" {};
    ttl = mkOption {
      type = nullOr int;
      default = null;
    };
    fqdn = mkOption {
      type = nullOr str;
      default = null;
    };
    subdomains = mkOption {
      type = nonEmptyListOf str;
    };
  };

  config = mkIf cfg.enable {
    services.avahi = {
      enable = true;
      publish = {
        enable = true;
        addresses = true;
        userServices = true;
      };
    };

    systemd.services = {
      avahi-daemon.wants = ["avahi-subdomains.service"];

      avahi-subdomains = {
        description = "avahi-subdomains";
        requires = ["avahi-daemon.service"];
        after = ["avahi-daemon.service"];
        partOf = ["avahi-daemon.service"];

        serviceConfig = {
          ExecStart = "${getExe cfg.package} ${ttlFlag} ${fqdnFlag} ${subdomainFlags}";
          Restart = "on-failure";
          RestartSec = 5;
        };
      };
    };
  };

  _file = ./module.nix;
  _class = "nixos";
}
