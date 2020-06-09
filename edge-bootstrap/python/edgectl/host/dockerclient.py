from __future__ import print_function
from io import BytesIO
import logging
import tarfile
import time
import docker
from edgectl.errors import EdgeError

class EdgeDockerClient(object):
    _DOCKER_INFO_OS_TYPE_KEY = 'OSType'

    def __init__(self):
        self._client = docker.DockerClient.from_env()
        params_dict = docker.utils.kwargs_from_env()
        base_url = None
        tls = None
        if params_dict:
            keys_list = list(params_dict.keys())
            if 'base_url' in keys_list:
                base_url = params_dict['base_url']
            if 'tls' in keys_list:
                tls = params_dict['tls']
        self._api_client = docker.APIClient(base_url=base_url, tls=tls)

    def check_availability(self):
        """
        API to check if docker is available

        Returns:
            True if docker is available.
            False otherwise
        """
        is_available = False
        try:
            self._client.info()
            is_available = True
        except docker.errors.APIError as ex:
            msg = 'Could not connect to docker daemon. {0}'.format(ex)
            logging.error(msg)

        return is_available

    def login(self, addr, uname, pword):
        logging.info('Logging into registry ' + addr \
                     + ' using username ' + uname)
        try:
            self._client.login(username=uname, password=pword, registry=addr)
        except docker.errors.APIError as ex:
            logging.error('Could not login to registry %s using username %s.', addr, uname)
            print(ex)
            raise

    def get_os_type(self):
        try:
            info = self._client.info()
            return info[self._DOCKER_INFO_OS_TYPE_KEY]
        except docker.errors.APIError as ex:
            raise EdgeError('Docker daemon returned error.', ex)

    def get_local_image_sha_id(self, image):
        """
        API to return the image sha id if it is available locally.

        Args:
            image (str): Name of image from which to retrieve it's id

        Returns:
            String containing the tag of the image
            None if the image is not available locally.
        """
        local_id = None
        try:
            logging.info('Checking if image exists locally: %s', image)
            inspect_dict = self._api_client.inspect_image(image)
            local_id = inspect_dict['Id']
            logging.info('Image exists locally. Id: %s', local_id)
        except docker.errors.APIError:
            logging.debug('Image not found locally: %s', image)

        return local_id

    def pull(self, image, username, password):
        """
        API to pull the latest binaries from the given image's repository.

        Args:
            image (str): Name of image to pull from its repository.
            username (str): Username to access image's repository.
                            None if no credentials are required.
            password (str): Password to access image's repository.
                            None if no credentials are required.

        Returns:
            True if a newer image was found and downloaded
            False otherwise
        """
        logging.info('Executing pull for: %s', image)
        is_updated = True
        old_id = None

        old_id = self.get_local_image_sha_id(image)
        if old_id is None:
            logging.info('Please note depending on network conditions and registry server' \
                         ' availability this may take a few minutes.')
        else:
            logging.info('Checking for newer tag for image: %s', image)

        try:
            auth_dict = None
            if username:
                auth_dict = {'username': username, 'password': password}
            self._client.images.pull(image, auth_config=auth_dict)
            logging.info('Completed pull for image: %s', image)
            if old_id is not None:
                inspect_dict = self._api_client.inspect_image(image)
                new_id = inspect_dict['Id']
                logging.debug('Newly pulled image id: %s', new_id)
                if new_id == old_id:
                    logging.info('Image is up to date.')
                    is_updated = False
                else:
                    logging.info('Pulled image with newer tag: %s', new_id)
        except docker.errors.APIError as ex:
            logging.error('Error inspecting image: %s. Error: %s', image, str(ex))
            raise

        return is_updated

    def get_container_image(self, container_name):
        container = self.get_container_by_name(container_name)
        if container is not None:
            return container.image
        return None

    def get_container_by_name(self, container_name):
        try:
            return self._client.containers.get(container_name)
        except docker.errors.NotFound:
            logging.debug('Could not find container %s', container_name)
            return None
        except docker.errors.APIError as ex:
            logging.error('Error when getting container: %s', container_name)
            print(ex)
            raise

    def start(self, container_name):
        logging.info('Starting container: ' + container_name)
        try:
            containers = self._client.containers.list(all=True)
            for container in containers:
                if container_name == container.name:
                    container.start()
        except docker.errors.APIError as ex:
            logging.error('Could not start container: %s', container_name)
            print(ex)
            raise

    def restart(self, container_name, timeout_int=5):
        logging.info('Restarting container: ' + container_name)
        try:
            containers = self._client.containers.list(all=True)
            for container in containers:
                if container_name == container.name:
                    container.restart(timeout=timeout_int)
        except docker.errors.APIError as ex:
            logging.error('Could not retart container: %s', container_name)
            print(ex)
            raise

    def stop(self, container_name):
        logging.info('Stopping container: ' + container_name)
        try:
            containers = self._client.containers.list(all=True)
            for container in containers:
                if container_name == container.name:
                    container.stop()
        except docker.errors.APIError as ex:
            logging.error('Could not stop container: %s', container_name)
            print(ex)
            raise

    def status(self, container_name):
        try:
            result = None
            containers = self._client.containers.list(all=True)
            for container in containers:
                if container_name == container.name:
                    result = container.status
            return result
        except docker.errors.APIError as ex:
            logging.error('Error while checking status for: %s', container_name)
            print(ex)
            raise

    def remove(self, container_name):
        logging.info('Removing container: ' + container_name)
        try:
            containers = self._client.containers.list(all=True)
            for container in containers:
                if container_name == container.name:
                    container.remove()
        except docker.errors.APIError as ex:
            logging.error('Could not remove container: %s' + container_name)
            print(ex)
            raise

    def stop_by_label(self, label):
        logging.info('Stopping containers by label: ' + label)
        try:
            filter_dict = {'label': label}
            containers = self._client.containers.list(all=True,
                                                      filters=filter_dict)
            for container in containers:
                container.stop()
        except docker.errors.APIError as ex:
            logging.error('Could not stop containers by label: %s', label)
            print(ex)
            raise
        return

    def remove_by_label(self, label):
        logging.info('Removing containers by label: ' + label)
        try:
            filter_dict = {'label': label}
            containers = self._client.containers.list(all=True,
                                                      filters=filter_dict)
            for container in containers:
                container.remove()
        except docker.errors.APIError as ex:
            logging.error('Could not remove containers by label: %s', label)
            print(ex)
            raise
        return

    def create_network(self, network_name):
        logging.info('Creating network: ' + network_name)
        create_network = False
        try:
            networks = self._client.networks.list(names=[network_name])
            if networks:
                num_networks = len(networks)
                if num_networks == 0:
                    create_network = True
            else:
                create_network = True
            if create_network:
                if self.get_os_type() == 'windows':
                    # default network type in Windows is nat
                    self._client.networks.create(network_name, driver="nat")
                else:
                    self._client.networks.create(network_name, driver="bridge")
        except docker.errors.APIError as ex:
            logging.error('Could not create docker network: %s' + network_name)
            print(ex)
            raise

    def create(self, image, container_name, detach_bool, env_dict, nw_name,
               ports_dict, volume_dict, log_config_dict, mounts_list,
               restart_policy_dict):
        try:
            logging.info('Executing docker create %s  name: %s  detach: %s' \
                         ' network: %s', image, container_name,
                         str(detach_bool), nw_name)
            for key in list(env_dict.keys()):
                logging.debug(' env: %s:%s', key, env_dict[key])
            for key in list(ports_dict.keys()):
                logging.debug(' port: %s:%s', key, str(ports_dict[key]))
            for key in list(volume_dict.keys()):
                logging.debug(' volume: %s:%s, %s', key,
                              volume_dict[key]['bind'], volume_dict[key]['mode'])
            if 'type' in list(log_config_dict.keys()):
                logging.debug(' logging driver: %s', log_config_dict['type'])
            if 'config' in list(log_config_dict.keys()):
                for key in list(log_config_dict['config'].keys()):
                    logging.debug(' log opt: %s:%s',
                                  key, log_config_dict['config'][key])
            self._client.containers.create(image,
                                           detach=detach_bool,
                                           environment=env_dict,
                                           name=container_name,
                                           network=nw_name,
                                           ports=ports_dict,
                                           volumes=volume_dict,
                                           log_config=log_config_dict,
                                           mounts=mounts_list,
                                           restart_policy=restart_policy_dict)
        except docker.errors.ContainerError as ex_ctr:
            logging.error('Container exited with errors: %s', container_name)
            print(ex_ctr)
            raise
        except docker.errors.ImageNotFound as ex_img:
            logging.error('Docker create failed. Image not found: %s', image)
            print(ex_img)
            raise
        except docker.errors.APIError as ex:
            logging.error('Docker create failed for image: %s', image)
            print(ex)
            raise

    def _get_volume_if_exists(self, name):
        logging.debug('Checking if volume exists: %s', name)
        volume = None
        try:
            volume = self._client.volumes.get(name)
        except docker.errors.NotFound:
            logging.debug('Volume does not exist: %s', name)
        except docker.errors.APIError as ex:
            logging.error('Docker volume get failed for: %s', name)
            print(ex)
            raise
        return volume

    def create_volume(self, name):
        try:
            volume = self._get_volume_if_exists(name)
            if volume:
                logging.info('Creating volume: %s', name)
                self._client.volumes.create(name)
        except docker.errors.APIError as ex:
            logging.error('Docker volume create failed for: %s', name)
            print(ex)
            raise

    def remove_volume(self, name, force=False):
        try:
            volume = self._get_volume_if_exists(name)
            if volume:
                logging.info('Removing volume: %s', name)
                volume.remove(force)
        except docker.errors.APIError as ex:
            logging.error('Docker volume remove failed for: %s', name)
            print(ex)
            raise

    def copy_file_to_volume(self,
                            container_name,
                            container_dest_file_name,
                            container_dest_dir_path,
                            host_src_file):
        tar_stream = BytesIO()
        container_tar_file = tarfile.TarFile(fileobj=tar_stream, mode='w')
        file_data = open(host_src_file, 'rb').read()
        tarinfo = tarfile.TarInfo(name=container_dest_file_name)
        tarinfo.size = len(file_data)
        tarinfo.mtime = time.time()
        tarinfo.mode = 0o444
        container_tar_file.addfile(tarinfo, BytesIO(file_data))
        container_tar_file.close()
        tar_stream.seek(0)
        container = self.get_container_by_name(container_name)
        container.put_archive(container_dest_dir_path, tar_stream)
