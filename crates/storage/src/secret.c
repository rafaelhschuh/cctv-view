#include <libsecret/secret.h>
#include <string.h>
static const SecretSchema schema = { "br.dev.schuh.cctv-view", SECRET_SCHEMA_NONE,
    { {"camera", SECRET_SCHEMA_ATTRIBUTE_STRING}, {NULL, 0} }, 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL };
int cctv_secret_store(const char *id, const char *value) {
    GError *error=NULL;
    int ok=secret_password_store_sync(&schema, SECRET_COLLECTION_DEFAULT, "CCTV View — câmera", value, NULL, &error, "camera", id, NULL);
    if(error) g_error_free(error);
    return ok;
}
char *cctv_secret_get(const char *id, int *failed) {
    GError *error=NULL;
    char *value=secret_password_lookup_sync(&schema,NULL,&error,"camera",id,NULL);
    *failed = error != NULL;
    if(error) g_error_free(error);
    return value;
}
int cctv_secret_delete(const char *id) {
    GError *error=NULL;
    secret_password_clear_sync(&schema,NULL,&error,"camera",id,NULL);
    if(error) { g_error_free(error); return 0; }
    return 1;
}
void cctv_secret_free(char *value) { secret_password_free(value); }
