resource_conf = configuration_data()
resource_conf.set('GRESOURCE_PATH', gresource_path)
resource_conf.set('APP_ID', application_id)

resources_xml = configure_file(
    input: 'resources.gresource.xml.in',
    output: 'resources.gresource.xml',
    configuration: resource_conf,
)

resources = gnome.compile_resources(
    'resources',
    resources_xml,
    gresource_bundle: true,
    source_dir: meson.current_source_dir(),
    install: true,
    install_dir: pkgdatadir,
)
