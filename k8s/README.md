You need to manage your own secrets in data-secret.yml. One option is to use http://git-secret.io to
setup your own file.



---
apiVersion: v1
kind: Secret
metadata:
  name: weaver-data
  labels:
    app: weaver
type: Opaque
data:
  storePassword: <YOUR BASE64 PASSWORD HERE>
