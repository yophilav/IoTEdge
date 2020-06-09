import json
import logging as log
import edgeconstants as EC
from edgeconfig import EdgeHostConfig
from edgeconfig import EdgeDeploymentConfigDocker
from edgeconfigparser import EdgeConfigParser

class EdgeConfigParserFile(EdgeConfigParser):

    def __init__(self, args):
        super(EdgeConfigParserFile, self).__init__(args, None)

    def parse(self, config_file=None):
        result = None
        try:
            if config_file is None:
                config_file = self._input_args.config_file
            with open(config_file, 'r') as edge_config:
                data = json.load(edge_config)
            config = EdgeHostConfig()
            config.schema_version = data[EC.SCHEMA_KEY]
            config.connection_string = data[EC.DEVICE_CONNECTION_STRING_KEY]
            config.home_dir = data[EC.HOMEDIR_KEY]
            config.hostname = data[EC.HOSTNAME_KEY]
            config.log_level = data[EC.EDGE_RUNTIME_LOG_LEVEL_KEY]
            certs_cfg_data = data[EC.SECURITY_KEY][EC.CERTS_KEY]
            config.security_option = certs_cfg_data[EC.CERTS_OPTION_KEY]
            if config.use_self_signed_certificates():
                ss_cert_data = certs_cfg_data[EC.SELFSIGNED_KEY]
                config.self_signed_cert_option_force_regen = \
                    ss_cert_data[EC.SELFSIGNED_FORCEREGEN_KEY]
            else:
                pre_install_cfg = certs_cfg_data[EC.PREINSTALLED_KEY]
                config.ca_cert_path = \
                    pre_install_cfg[EC.PREINSTALLED_DEVICE_CA_CERT_KEY]
                config.edge_server_cert_path = \
                    pre_install_cfg[EC.PREINSTALLED_SERVER_CERT_KEY]

            docker_cfg = None
            deployment_type = data[EC.DEPLOYMENT_KEY][EC.DEPLOYMENT_TYPE_KEY]
            if deployment_type == EC.DEPLOYMENT_DOCKER_KEY:
                docker_cfg = data[EC.DEPLOYMENT_KEY][EC.DEPLOYMENT_DOCKER_KEY]
                deploy_cfg = EdgeDeploymentConfigDocker()
                deploy_cfg.uri = docker_cfg[EC.DOCKER_URI_KEY]
                deploy_cfg.edge_image = docker_cfg[EC.EDGE_RUNTIME_IMAGE_KEY]
                for reg in docker_cfg[EC.REGISTRIES_KEY]:
                    deploy_cfg.add_registry(reg[EC.REGISTRY_ADDRESS_KEY],
                                            reg[EC.REGISTRY_USERNAME_KEY],
                                            reg[EC.REGISTRY_PASSWORD_KEY])
                docker_log_cfg = docker_cfg[EC.DOCKER_LOGGING_OPTIONS_KEY]
                deploy_cfg.logging_driver = \
                    docker_log_cfg[EC.DOCKER_LOGGING_DRIVER_KEY]
                log_opts = docker_log_cfg[EC.DOCKER_LOGGING_DRIVER_OPTIONS_KEY]
                for opt_key, opt_val in list(log_opts.items()):
                    deploy_cfg.add_logging_option(opt_key, opt_val)

            if docker_cfg is None:
                raise ValueError('Unsupported Deployment Type:' \
                                 + deployment_type)
            config.deployment_config = deploy_cfg
            self._deployment_type = deployment_type
            result = config
        except OSError as ex_os:
            log.error('Error Observed When Reading Edge Config File: ' \
                      + config_file + '. Errno ' + str(ex_os.errno) \
                      + ', Error:' + ex_os.strerror)
            raise
        except ValueError as ex_value:
            log.error('Error Observed When Parsing Edge Config File: ' \
                      + config_file)
            raise

        return result
