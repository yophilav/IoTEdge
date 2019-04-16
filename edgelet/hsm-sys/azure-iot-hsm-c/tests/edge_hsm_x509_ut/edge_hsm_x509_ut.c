// Copyright (c) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.

#include <stdlib.h>
#include <string.h>
#include <stddef.h>

//#############################################################################
// Memory allocator test hooks
//#############################################################################

static void* test_hook_gballoc_malloc(size_t size)
{
    return malloc(size);
}

static void* test_hook_gballoc_calloc(size_t num, size_t size)
{
    return calloc(num, size);
}

static void* test_hook_gballoc_realloc(void* ptr, size_t size)
{
    return realloc(ptr, size);
}

static void test_hook_gballoc_free(void* ptr)
{
    free(ptr);
}

#include "testrunnerswitcher.h"
#include "umock_c.h"
#include "umock_c_negative_tests.h"
#include "umocktypes_charptr.h"

//#############################################################################
// Declare and enable MOCK definitions
//#############################################################################
#include "hsm_certificate_props.h"
#include "hsm_client_data.h"
#include "certificate_info.h"

#define ENABLE_MOCKS
#include "azure_c_shared_utility/gballoc.h"
#include "azure_c_shared_utility/crt_abstractions.h"
#include "hsm_utils.h"

// interface mocks
MOCKABLE_FUNCTION(, int, hsm_client_crypto_init);
MOCKABLE_FUNCTION(, void, hsm_client_crypto_deinit);
MOCKABLE_FUNCTION(, const HSM_CLIENT_CRYPTO_INTERFACE*, hsm_client_crypto_interface);
MOCKABLE_FUNCTION(, const char*, hsm_get_device_ca_alias);

// crypto API mocks
MOCKABLE_FUNCTION(, HSM_CLIENT_HANDLE, mocked_hsm_client_crypto_create);
MOCKABLE_FUNCTION(, void, mocked_hsm_client_crypto_destroy, HSM_CLIENT_HANDLE, handle);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_get_random_bytes, HSM_CLIENT_HANDLE, handle, unsigned char*, buffer, size_t, num);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_create_master_encryption_key, HSM_CLIENT_HANDLE, handle);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_destroy_master_encryption_key, HSM_CLIENT_HANDLE, handle);
MOCKABLE_FUNCTION(, CERT_INFO_HANDLE, mocked_hsm_client_create_certificate, HSM_CLIENT_HANDLE, handle, CERT_PROPS_HANDLE, certificate_props);
MOCKABLE_FUNCTION(, CERT_INFO_HANDLE, mocked_hsm_client_crypto_get_certificate, HSM_CLIENT_HANDLE, handle, const char*, alias);
MOCKABLE_FUNCTION(, void, mocked_hsm_client_destroy_certificate, HSM_CLIENT_HANDLE, handle, const char*, alias);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_encrypt_data, HSM_CLIENT_HANDLE, handle, const SIZED_BUFFER*, identity, const SIZED_BUFFER*, plaintext, const SIZED_BUFFER*, init_vector, SIZED_BUFFER*, ciphertext);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_decrypt_data, HSM_CLIENT_HANDLE, handle, const SIZED_BUFFER*, identity, const SIZED_BUFFER*, ciphertext, const SIZED_BUFFER*, init_vector, SIZED_BUFFER*, plaintext);
MOCKABLE_FUNCTION(, int, mocked_hsm_client_crypto_sign_with_private_key, HSM_CLIENT_HANDLE, handle, const char*, alias, const unsigned char*, data, size_t, data_size, unsigned char**, digest, size_t*, digest_size);
MOCKABLE_FUNCTION(, void, mocked_hsm_client_free_buffer, void*, buffer);
MOCKABLE_FUNCTION(, CERT_INFO_HANDLE, mocked_hsm_client_get_trust_bundle, HSM_CLIENT_HANDLE, handle);

// certificate props API mocks
MOCKABLE_FUNCTION(, CERT_PROPS_HANDLE, cert_properties_create);
MOCKABLE_FUNCTION(, void, cert_properties_destroy, CERT_PROPS_HANDLE, handle);
MOCKABLE_FUNCTION(, int, set_validity_seconds, CERT_PROPS_HANDLE, handle, uint64_t, validity_secs);
MOCKABLE_FUNCTION(, int, set_common_name, CERT_PROPS_HANDLE, handle, const char*, common_name);
MOCKABLE_FUNCTION(, int, set_issuer_alias, CERT_PROPS_HANDLE, handle, const char*, issuer_alias);
MOCKABLE_FUNCTION(, int, set_alias, CERT_PROPS_HANDLE, handle, const char*, alias);
MOCKABLE_FUNCTION(, int, set_certificate_type, CERT_PROPS_HANDLE, handle, CERTIFICATE_TYPE, type);

MOCKABLE_FUNCTION(, CERT_INFO_HANDLE, certificate_info_create, const char*, certificate, const void*, private_key, size_t, priv_key_len, PRIVATE_KEY_TYPE, pk_type);
MOCKABLE_FUNCTION(, const char*, get_alias, CERT_PROPS_HANDLE, handle);
MOCKABLE_FUNCTION(, const char*, get_issuer_alias, CERT_PROPS_HANDLE, handle);
MOCKABLE_FUNCTION(, int64_t, certificate_info_get_valid_to, CERT_INFO_HANDLE, handle);
MOCKABLE_FUNCTION(, void, certificate_info_destroy, CERT_INFO_HANDLE, handle);
MOCKABLE_FUNCTION(, const char*, certificate_info_get_certificate, CERT_INFO_HANDLE, handle);
MOCKABLE_FUNCTION(, const void*, certificate_info_get_private_key, CERT_INFO_HANDLE, handle, size_t*, priv_key_len);

#undef ENABLE_MOCKS

//#############################################################################
// Interface(s) under test
//#############################################################################
#include "hsm_client_data.h"

//#############################################################################
// Test defines and data
//#############################################################################

#define TEST_HSM_CLIENT_HANDLE (HSM_CLIENT_HANDLE)0x1000
#define TEST_CERT_INFO_HANDLE (CERT_INFO_HANDLE)0x1001
#define TEST_TRUST_BUNDLE_CERT_INFO_HANDLE (CERT_INFO_HANDLE)0x1004
#define TEST_CERT_PROPS_HANDLE (CERT_PROPS_HANDLE)0x1005

DEFINE_ENUM_STRINGS(UMOCK_C_ERROR_CODE, UMOCK_C_ERROR_CODE_VALUES)

static TEST_MUTEX_HANDLE g_testByTest;
static TEST_MUTEX_HANDLE g_dllByDll;

const char* TEST_ALIAS_STRING = "test_alias";
const char* TEST_ISSUER_ALIAS_STRING = "test_issuer_alias";

static const HSM_CLIENT_CRYPTO_INTERFACE mocked_hsm_client_crypto_interface =
{
    mocked_hsm_client_crypto_create,
    mocked_hsm_client_crypto_destroy,
    mocked_hsm_client_get_random_bytes,
    mocked_hsm_client_create_master_encryption_key,
    mocked_hsm_client_destroy_master_encryption_key,
    mocked_hsm_client_create_certificate,
    mocked_hsm_client_destroy_certificate,
    mocked_hsm_client_encrypt_data,
    mocked_hsm_client_decrypt_data,
    mocked_hsm_client_get_trust_bundle,
    mocked_hsm_client_free_buffer,
    mocked_hsm_client_crypto_sign_with_private_key,
    mocked_hsm_client_crypto_get_certificate
};

//#############################################################################
// Mocked functions test hooks
//#############################################################################

static void test_hook_on_umock_c_error(UMOCK_C_ERROR_CODE error_code)
{
    char temp_str[256];
    (void)snprintf(temp_str, sizeof(temp_str), "umock_c reported error :%s",
                   ENUM_TO_STRING(UMOCK_C_ERROR_CODE, error_code));
    ASSERT_FAIL(temp_str);
}

static const HSM_CLIENT_CRYPTO_INTERFACE* test_hook_hsm_client_crypto_interface(void)
{
    return &mocked_hsm_client_crypto_interface;
}

static int test_hook_hsm_client_crypto_init()
{
    return 0;
}

static HSM_CLIENT_HANDLE test_hook_hsm_client_crypto_create(void)
{
    return TEST_HSM_CLIENT_HANDLE;
}

static void test_hook_hsm_client_crypto_destroy(HSM_CLIENT_HANDLE handle)
{
    (void)handle;
}

static int test_hook_hsm_client_get_random_bytes(HSM_CLIENT_HANDLE handle, unsigned char* buffer, size_t num)
{
    (void)handle;
    (void)buffer;
    (void)num;
    return 0;
}

static int test_hook_hsm_client_create_master_encryption_key(HSM_CLIENT_HANDLE handle)
{
    (void)handle;
    return 0;
}

static int test_hook_hsm_client_destroy_master_encryption_key(HSM_CLIENT_HANDLE handle)
{
    (void)handle;
    return 0;
}

static CERT_INFO_HANDLE test_hook_hsm_client_create_certificate(HSM_CLIENT_HANDLE handle, CERT_PROPS_HANDLE certificate_props)
{
    (void)handle;
    (void)certificate_props;
    return TEST_CERT_INFO_HANDLE;
}

static CERT_INFO_HANDLE test_hook_hsm_client_crypto_get_certificate(HSM_CLIENT_HANDLE handle, const char* alias)
{
    (void)handle;
    (void)alias;
    return TEST_CERT_INFO_HANDLE;
}

static void test_hook_hsm_client_destroy_certificate(HSM_CLIENT_HANDLE handle, const char* alias)
{
    (void)handle;
    (void)alias;
}

static int test_hook_hsm_client_encrypt_data(HSM_CLIENT_HANDLE handle, const SIZED_BUFFER* identity, const SIZED_BUFFER* plaintext, const SIZED_BUFFER* init_vector, SIZED_BUFFER* ciphertext)
{
    (void)handle;
    (void)identity;
    (void)plaintext;
    (void)init_vector;
    (void)ciphertext;
    return 0;
}

static int test_hook_hsm_client_decrypt_data(HSM_CLIENT_HANDLE handle, const SIZED_BUFFER* identity, const SIZED_BUFFER* ciphertext, const SIZED_BUFFER* init_vector, SIZED_BUFFER* plaintext)
{
    (void)handle;
    (void)identity;
    (void)ciphertext;
    (void)init_vector;
    (void)plaintext;
    return 0;
}

static CERT_INFO_HANDLE test_hook_hsm_client_get_trust_bundle(HSM_CLIENT_HANDLE handle)
{
    (void)handle;
    return TEST_TRUST_BUNDLE_CERT_INFO_HANDLE;
}

static void test_hook_hsm_client_free_buffer(void* buffer)
{
    (void)buffer;
}

static int test_hook_hsm_client_crypto_sign_with_private_key(HSM_CLIENT_HANDLE handle, const char* alias, const unsigned char* data, size_t data_size, unsigned char** digest, size_t* digest_size)
{
    (void)handle;
    (void)alias;
    (void)data;
    (void)data_size;
    (void)digest;
    (void)digest_size;
    return 0;
}

static CERT_INFO_HANDLE test_hook_certificate_info_create
(
    const char* certificate,
    const void* private_key,
    size_t priv_key_len,
    PRIVATE_KEY_TYPE pk_type
)
{
    (void)certificate;
    (void)private_key;
    (void)priv_key_len;
    (void)pk_type;
    return TEST_CERT_INFO_HANDLE;
}

//#############################################################################
// Test cases
//#############################################################################

BEGIN_TEST_SUITE(edge_hsm_x509_unittests)

    TEST_SUITE_INITIALIZE(TestClassInitialize)
    {
        TEST_INITIALIZE_MEMORY_DEBUG(g_dllByDll);
        g_testByTest = TEST_MUTEX_CREATE();
        ASSERT_IS_NOT_NULL(g_testByTest);

        umock_c_init(test_hook_on_umock_c_error);

        REGISTER_UMOCK_ALIAS_TYPE(HSM_CLIENT_HANDLE, void*);
        REGISTER_UMOCK_ALIAS_TYPE(TEST_CERT_INFO_HANDLE, void*);
        REGISTER_UMOCK_ALIAS_TYPE(CERT_PROPS_HANDLE, void*);

        ASSERT_ARE_EQUAL(int, 0, umocktypes_charptr_register_types() );

        REGISTER_GLOBAL_MOCK_HOOK(gballoc_malloc, test_hook_gballoc_malloc);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(gballoc_malloc, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(gballoc_calloc, test_hook_gballoc_calloc);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(gballoc_calloc, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(gballoc_realloc, test_hook_gballoc_realloc);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(gballoc_realloc, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(gballoc_free, test_hook_gballoc_free);

        REGISTER_GLOBAL_MOCK_HOOK(hsm_client_crypto_init, test_hook_hsm_client_crypto_init);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(hsm_client_crypto_init, 1);

        REGISTER_GLOBAL_MOCK_HOOK(hsm_client_crypto_interface, test_hook_hsm_client_crypto_interface);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(hsm_client_crypto_interface, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_crypto_create, test_hook_hsm_client_crypto_create);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_crypto_create, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_crypto_destroy, test_hook_hsm_client_crypto_destroy);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_get_random_bytes, test_hook_hsm_client_get_random_bytes);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_get_random_bytes, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_create_master_encryption_key, test_hook_hsm_client_create_master_encryption_key);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_create_master_encryption_key, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_destroy_master_encryption_key, test_hook_hsm_client_destroy_master_encryption_key);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_destroy_master_encryption_key, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_create_certificate, test_hook_hsm_client_create_certificate);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_create_certificate, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_destroy_certificate, test_hook_hsm_client_destroy_certificate);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_encrypt_data, test_hook_hsm_client_encrypt_data);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_encrypt_data, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_decrypt_data, test_hook_hsm_client_decrypt_data);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_decrypt_data, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_get_trust_bundle, test_hook_hsm_client_get_trust_bundle);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_get_trust_bundle, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_free_buffer, test_hook_hsm_client_free_buffer);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_crypto_sign_with_private_key, test_hook_hsm_client_crypto_sign_with_private_key);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_crypto_sign_with_private_key, 1);

        REGISTER_GLOBAL_MOCK_HOOK(mocked_hsm_client_crypto_get_certificate, test_hook_hsm_client_crypto_get_certificate);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(mocked_hsm_client_crypto_get_certificate, NULL);

        REGISTER_GLOBAL_MOCK_HOOK(certificate_info_create, test_hook_certificate_info_create);
        REGISTER_GLOBAL_MOCK_FAIL_RETURN(certificate_info_create, NULL);
    }

    TEST_SUITE_CLEANUP(TestClassCleanup)
    {
        umock_c_deinit();

        TEST_MUTEX_DESTROY(g_testByTest);
        TEST_DEINITIALIZE_MEMORY_DEBUG(g_dllByDll);
    }

    TEST_FUNCTION_INITIALIZE(TestMethodInitialize)
    {
        if (TEST_MUTEX_ACQUIRE(g_testByTest))
        {
            ASSERT_FAIL("Mutex is ABANDONED. Failure in test framework.");
        }

        umock_c_reset_all_calls();
    }

    TEST_FUNCTION_CLEANUP(TestMethodCleanup)
    {
        TEST_MUTEX_RELEASE(g_testByTest);
    }

    /**
     * Test function for API
     *   hsm_client_x509_init
    */
    TEST_FUNCTION(hsm_client_x509_init_success)
    {
        //arrange
        int status;
        EXPECTED_CALL(hsm_client_crypto_init());

        // act
        status = hsm_client_x509_init();

        // assert
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        hsm_client_x509_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_init
    */
    TEST_FUNCTION(hsm_client_x509_multi_init_success)
    {
        //arrange
        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        umock_c_reset_all_calls();

        // act
        status = hsm_client_x509_init();

        // assert
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        hsm_client_x509_deinit();
        hsm_client_x509_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_crypto_init
    */
    TEST_FUNCTION(hsm_client_x509_init_negative)
    {
        //arrange
        int test_result = umock_c_negative_tests_init();
        ASSERT_ARE_EQUAL(int, 0, test_result);

        EXPECTED_CALL(hsm_client_crypto_init());
        umock_c_negative_tests_snapshot();

        for (size_t i = 0; i < umock_c_negative_tests_call_count(); i++)
        {
            int status;
            umock_c_negative_tests_reset();
            umock_c_negative_tests_fail_call(i);

            // act
            status = hsm_client_x509_init();

            // assert
            ASSERT_ARE_NOT_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        }

        //cleanup
        umock_c_negative_tests_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_interface
    */
    TEST_FUNCTION(hsm_client_x509_interface_success)
    {
        //arrange

        // act
        const HSM_CLIENT_X509_INTERFACE* result = hsm_client_x509_interface();

        // assert
        ASSERT_IS_NOT_NULL(result, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_x509_create, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_x509_destroy, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_get_cert, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_get_key, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_get_common_name, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_free_buffer, "Line:" TOSTRING(__LINE__));
        ASSERT_IS_NOT_NULL(result->hsm_client_sign_with_private_key, "Line:" TOSTRING(__LINE__));

        //cleanup
    }

    /**
     * Test function for API
     *   hsm_client_x509_create
    */
    TEST_FUNCTION(hsm_client_x509_create_success)
    {
        //arrange
        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        umock_c_reset_all_calls();
        EXPECTED_CALL(hsm_client_crypto_interface());
        EXPECTED_CALL(mocked_hsm_client_crypto_create());

        // act
        CERT_INFO_HANDLE handle = interface->hsm_client_x509_create();

        // assert
        ASSERT_IS_NOT_NULL(handle, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        interface->hsm_client_x509_destroy(handle);
        hsm_client_x509_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_create
    */
    TEST_FUNCTION(hsm_client_x509_create_without_init_fails)
    {
        //arrange
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();

        // act
        CERT_INFO_HANDLE handle = interface->hsm_client_x509_create();

        // assert
        ASSERT_IS_NULL(handle, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
    }

    /**
     * Test function for API
     *   hsm_client_x509_create
    */
    TEST_FUNCTION(hsm_client_x509_create_negative)
    {
        //arrange
        int test_result = umock_c_negative_tests_init();
        ASSERT_ARE_EQUAL(int, 0, test_result);

        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        umock_c_reset_all_calls();
        EXPECTED_CALL(hsm_client_crypto_interface());
        EXPECTED_CALL(mocked_hsm_client_crypto_create());

        umock_c_negative_tests_snapshot();

        for (size_t i = 0; i < umock_c_negative_tests_call_count(); i++)
        {
            umock_c_negative_tests_reset();
            umock_c_negative_tests_fail_call(i);

            // act
            CERT_INFO_HANDLE handle = interface->hsm_client_x509_create();

            // assert
            ASSERT_IS_NULL(handle, "Line:" TOSTRING(__LINE__));
        }

        //cleanup
        hsm_client_x509_deinit();
        umock_c_negative_tests_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_destroy
    */
    TEST_FUNCTION(hsm_client_x509_destroy_invalid_param_does_nothing)
    {
        //arrange
        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        umock_c_reset_all_calls();

        // act
        interface->hsm_client_x509_destroy(NULL);

        // assert
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        hsm_client_x509_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_destroy
    */
    TEST_FUNCTION(hsm_client_x509_destroy_success)
    {
        //arrange
        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        CERT_INFO_HANDLE handle = interface->hsm_client_x509_create();
        ASSERT_IS_NOT_NULL(handle, "Line:" TOSTRING(__LINE__));
        umock_c_reset_all_calls();
        EXPECTED_CALL(hsm_client_crypto_interface());
        STRICT_EXPECTED_CALL(mocked_hsm_client_crypto_destroy(handle));

        // act
        interface->hsm_client_x509_destroy(handle);

        // assert

        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        hsm_client_x509_deinit();
    }

    /**
     * Test function for API
     *   hsm_client_x509_destroy
    */
    TEST_FUNCTION(hsm_client_x509_destroy_without_does_nothing)
    {
        //arrange
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        umock_c_reset_all_calls();

        // act
        interface->hsm_client_x509_destroy(TEST_CERT_INFO_HANDLE);

        // assert
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
    }

    /**
     * Test function for API
     *   hsm_client_get_cert
    */
    TEST_FUNCTION(hsm_client_get_cert_success)
    {
        //arrange
        int status;
        status = hsm_client_x509_init();
        ASSERT_ARE_EQUAL(int, 0, status, "Line:" TOSTRING(__LINE__));
        const HSM_CLIENT_X509_INTERFACE* interface = hsm_client_x509_interface();
        umock_c_reset_all_calls();
        EXPECTED_CALL(hsm_client_crypto_interface());
        EXPECTED_CALL(mocked_hsm_client_crypto_create());

        // act
        CERT_INFO_HANDLE handle = interface->hsm_client_x509_create();

        // assert
        ASSERT_IS_NOT_NULL(handle, "Line:" TOSTRING(__LINE__));
        ASSERT_ARE_EQUAL(char_ptr, umock_c_get_expected_calls(), umock_c_get_actual_calls(), "Line:" TOSTRING(__LINE__));

        //cleanup
        interface->hsm_client_x509_destroy(handle);
        hsm_client_x509_deinit();
    }

END_TEST_SUITE(edge_hsm_x509_unittests)
