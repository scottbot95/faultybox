{ flake }:
{ config, lib, pkgs, ... }:
let
  cfg = config.services.faultybox;
in with lib; {
  options.services.faultybox = {
    enable = mkEnableOption "FaultyBox Game server";
    package = mkOption {
      type = types.package;
      default = flake.packages.${pkgs.system}.faultybox;
      defaultText = "flake.packages.\${system}.faultybox";
      description = "Package to use for FaultyBox service. Allows customizing version";
    };
    port = mkOption {
      type = types.port;
      default = 8080;
      description = "Port to bind game server to";
    };
    address = mkOption {
      type = types.str;
      default = "0.0.0.0";
      description = "Address to bind game server to";
    };
    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to open \${cfg.port} on the local firewall";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.faultybox = {
      description = "FaultyBox Game server";

      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      restartIfChanged = true;

      serviceConfig = {
        DynamicUser = true;
        ExecStart = "${cfg.package}/bin/server --addr ${cfg.address} --port ${toString cfg.port}";
        Restart = "always";
      };
    };

    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = [ 8080 ];
      allowedUDPPorts = [ 8080 ];
    };
  };
}