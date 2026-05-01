# F5 Programmability Demos: May 2026

These are a collection of example programs and demos from F5's Programmability
team. They are intended to be used by select F5 customers.

These programs are to run on an early-access version of F5's dataplanes with
Wasm Programmability enabled, provided to select customers and partners on the
[UDF](https://udf.f5.com/info). If you're interested in access, please
inqiurire with your F5 sales representative - I am unable to grant anyone
access directly.

## UDF Contents

To start, you'll need to deploy the UDF Blueprint [Programmability Demos May 2026](https://udf.f5.com/b/a6000fc9-9d73-47c2-a151-279917225a80#documentation).

On your deployment, select "Components", then under "Systems" there is a
single Ubuntu system. Click "DETAILS" on that system. You will then have a set
of buttons for accessing different services running on the Ubuntu host:

![UDF Buttons](.images/udf-buttons.png)


### Development environment: Visual Studio Code

This Ubuntu host provides a browser-based Visual Studio Code development
environment, where this repository is cloned as the workspace. You can open
this by clicking the button labeled "VSCODE".

When prompted, tell VS Code you trust the authors of the workspace.

Once in VS Code, there are a set of folders in the workspace, corresponding
to the folders in this repository. Each contains a README.md describing the
contents. Use the right click "Open Preview" on these README.md files to
render them nicely:

![VSCode Open Preview](.images/vscode-open-preview.png)

Each folder has a set of Tasks configured to be run by VS Code for building
and deploying the example.

### NGINX with Wasm Programmability


### BIG-IP with Wasm Programmability


### Wasm Control Plane: Platypus

The UDF loads Wasm programs into each dataplane using `platypus`, a prototype
control plane implementation for Wasm Programmability. Platypus provides an
web API, as well as a web browser frontend, for managing which Wasm services
are running on the associated dataplane.

Platypus is not intended to represent what a production control plane will
look like - its a minimum viable prototype that is just for early access
experimentation.

The platypus instance for NGINX is available in the UDF by clicking the button
labeled `NGINX WASM SERVICE MANAGER`. Internally to the UDF deployent, it is
running at `10.1.1.4:9000`.

The platypus instance for BIG-IP is available in the UDF by clicking the button
labeled `BIG-IP WASM SERVICE MANAGER`. Internally to the UDF deployent, it is
running at `10.1.1.4:9001`.



## Author

Pat Hickey

Sr Principal Software Development Engineer

`p.hickey@f5.com`

With help from: Daniel Edgar, Javier Evans, Oscar Spencer, Chris Fallin, and
Nick Fitzgerald


