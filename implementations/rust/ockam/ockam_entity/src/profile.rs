/// Profile is an abstraction responsible for keeping, verifying and modifying
/// user's data (mainly - public keys). It is used to create new keys, rotate and revoke them.
/// Public keys together with metadata will be organised into events chain, corresponding
/// secret keys will be saved into the given Vault implementation. Events chain and corresponding
/// secret keys are what fully determines Profile.
///
///
/// # Examples
///
/// Create a [`Profile`]. Add and rotate keys.
/// TODO
///
/// Authentication using [`Profile`]. In following example Bob authenticates Alice.
/// TODO
///
/// Update [`Profile`] and send changes to other parties. In following example Alice rotates
/// her key and sends corresponding [`Profile`] changes to Bob.
/// TODO
///
use crate::{
    traits::Verifier, AuthenticationProof, BbsCredential, Changes, Contact, Credential,
    CredentialAttribute, CredentialFragment1, CredentialFragment2, CredentialOffer,
    CredentialPresentation, CredentialProof, CredentialPublicKey, CredentialRequest,
    CredentialRequestFragment, CredentialSchema, Entity, EntityCredential, Handle, Holder,
    Identity, IdentityRequest, IdentityResponse, Issuer, Lease, OfferId, PresentationManifest,
    ProfileChangeEvent, ProfileIdentifier, ProofRequestId, SecureChannels, SigningPublicKey,
    TrustPolicy, TTL,
};
use ockam_core::async_trait::async_trait;
use ockam_core::compat::{boxed::Box, string::String, vec::Vec};
use ockam_core::traits::AsyncClone;
use ockam_core::{Address, Result, Route};
use ockam_vault::{PublicKey, Secret};
use signature_bls::SecretKey;

#[derive(Clone)]
pub struct Profile {
    id: ProfileIdentifier,
    handle: Handle,
}

impl From<Profile> for Entity {
    fn from(p: Profile) -> Entity {
        Entity::new(p.handle.clone(), Some(p.id.clone()))
    }
}

#[async_trait]
impl AsyncClone for Profile {
    async fn async_clone(&self) -> Profile {
        Profile {
            id: self.id.clone(),
            handle: self.handle.async_clone().await
        }
    }
}

impl Profile {
    pub fn new<I: Into<ProfileIdentifier>>(id: I, handle: Handle) -> Self {
        let id = id.into();
        Profile { id, handle }
    }

    pub async fn async_entity(&self) -> Entity {
        //Entity::async_from(self.async_clone().await).await
        Entity::new(self.handle.async_clone().await, Some(self.id.clone()))
    }

    pub fn entity(&self) -> Entity {
        Entity::from(self.clone())
    }

    pub fn call(&self, req: IdentityRequest) -> Result<IdentityResponse> {
        self.handle.call(req)
    }

    pub fn cast(&self, req: IdentityRequest) -> Result<()> {
        self.handle.cast(req)
    }
}

impl Profile {
    /// Sha256 of that value is used as previous event id for first event in a [`Profile`]
    pub const NO_EVENT: &'static [u8] = "OCKAM_NO_EVENT".as_bytes();
    /// Label for [`Profile`] update key
    pub const PROFILE_UPDATE: &'static str = "OCKAM_PUK";
    /// Label for key used to issue credentials
    pub const CREDENTIALS_ISSUE: &'static str = "OCKAM_CIK";
    /// Current version of change structure
    pub const CURRENT_CHANGE_VERSION: u8 = 1;
}

#[async_trait]
impl Identity for Profile {
    fn identifier(&self) -> Result<ProfileIdentifier> {
        self.entity().identifier()
    }

    async fn async_identifier(&self) -> Result<ProfileIdentifier> {
        self.async_entity().await.async_identifier().await
    }

    fn create_key<S: Into<String> + Send + 'static>(&mut self, label: S) -> Result<()> {
        self.entity().create_key(label)
    }

    async fn async_create_key<S: Into<String> + Send + 'static>(&mut self, label: S) -> Result<()> {
        self.entity().async_create_key(label).await
    }

    fn rotate_profile_key(&mut self) -> Result<()> {
        self.entity().rotate_profile_key()
    }

    fn get_profile_secret_key(&self) -> Result<Secret> {
        self.entity().get_profile_secret_key()
    }

    fn get_secret_key<S: Into<String>>(&self, label: S) -> Result<Secret> {
        self.entity().get_secret_key(label)
    }

    fn get_profile_public_key(&self) -> Result<PublicKey> {
        self.entity().get_profile_public_key()
    }

    fn get_public_key<S: Into<String>>(&self, label: S) -> Result<PublicKey> {
        self.entity().get_public_key(label)
    }

    fn create_auth_proof<S: AsRef<[u8]>>(&mut self, state_slice: S) -> Result<AuthenticationProof> {
        self.entity().create_auth_proof(state_slice)
    }

    async fn async_create_auth_proof<S: AsRef<[u8]> + Send + Sync>(&mut self, state_slice: S) -> Result<AuthenticationProof> {
        self.async_entity().await.async_create_auth_proof(state_slice).await
    }

    fn verify_auth_proof<S: AsRef<[u8]>, P: AsRef<[u8]>>(
        &mut self,
        state_slice: S,
        peer_id: &ProfileIdentifier,
        proof_slice: P,
    ) -> Result<bool> {
        self.entity()
            .verify_auth_proof(state_slice, peer_id, proof_slice)
    }

    async fn async_verify_auth_proof<S: AsRef<[u8]>  + Send + Sync, P: AsRef<[u8]> + Send + Sync>(
        &mut self,
        state_slice: S,
        peer_id: &ProfileIdentifier,
        proof_slice: P,
    ) -> Result<bool> {
        self.async_entity().await
            .async_verify_auth_proof(state_slice, peer_id, proof_slice).await
    }

    fn add_change(&mut self, change_event: ProfileChangeEvent) -> Result<()> {
        self.entity().add_change(change_event)
    }

    fn get_changes(&self) -> Result<Changes> {
        self.entity().get_changes()
    }

    async fn async_get_changes(&self) -> Result<Changes> {
        self.async_entity().await.async_get_changes().await
    }

    fn verify_changes(&mut self) -> Result<bool> {
        self.entity().verify_changes()
    }

    fn get_contacts(&self) -> Result<Vec<Contact>> {
        self.entity().get_contacts()
    }

    fn as_contact(&mut self) -> Result<Contact> {
        let changes = self.get_changes()?;
        Ok(Contact::new(self.id.clone(), changes))
    }

    async fn async_as_contact(&mut self) -> Result<Contact> {
        let changes = self.async_get_changes().await?;
        Ok(Contact::new(self.id.clone(), changes))
    }

    fn get_contact(&mut self, contact_id: &ProfileIdentifier) -> Result<Option<Contact>> {
        self.entity().get_contact(contact_id)
    }

    async fn async_get_contact(&mut self, contact_id: &ProfileIdentifier) -> Result<Option<Contact>> {
        self.async_entity().await.async_get_contact(contact_id).await
    }

    fn verify_contact<C: Into<Contact>>(&mut self, contact: C) -> Result<bool> {
        self.entity().verify_contact(contact)
    }

    async fn async_verify_contact<C: Into<Contact> + Send>(&mut self, contact: C) -> Result<bool> {
        self.async_entity().await.async_verify_contact(contact).await
    }

    fn verify_and_add_contact<C: Into<Contact>>(&mut self, contact: C) -> Result<bool> {
        self.entity().verify_and_add_contact(contact)
    }

    async fn async_verify_and_add_contact<C: Into<Contact> + Send>(&mut self, contact: C) -> Result<bool> {
        self.async_entity().await.async_verify_and_add_contact(contact).await
    }

    fn verify_and_update_contact<C: AsRef<[ProfileChangeEvent]>>(
        &mut self,
        contact_id: &ProfileIdentifier,
        change_events: C,
    ) -> Result<bool> {
        self.entity()
            .verify_and_update_contact(contact_id, change_events)
    }

    fn get_lease(
        &self,
        lease_manager_route: &Route,
        org_id: impl ToString,
        bucket: impl ToString,
        ttl: TTL,
    ) -> Result<Lease> {
        self.entity()
            .get_lease(lease_manager_route, org_id, bucket, ttl)
    }

    fn revoke_lease(&mut self, lease_manager_route: &Route, lease: Lease) -> Result<()> {
        self.entity().revoke_lease(lease_manager_route, lease)
    }
}

#[async_trait]
impl SecureChannels for Profile {
    fn create_secure_channel_listener(
        &mut self,
        address: impl Into<Address> + Send,
        trust_policy: impl TrustPolicy,
    ) -> Result<()> {
        self.entity()
            .create_secure_channel_listener(address, trust_policy)
    }

    fn create_secure_channel(
        &mut self,
        route: impl Into<Route> + Send,
        trust_policy: impl TrustPolicy,
    ) -> Result<Address> {
        self.entity().create_secure_channel(route, trust_policy)
    }

    async fn async_create_secure_channel_listener<A>(
        &mut self,
        address: A,
        trust_policy: impl TrustPolicy,
    ) -> Result<()>
    where
        A: Into<Address> + Send
    {
        self.entity()
            .async_create_secure_channel_listener(address, trust_policy).await
    }

    async fn async_create_secure_channel<R>(
        &mut self,
        route: R,
        trust_policy: impl TrustPolicy,
    ) -> Result<Address>
    where
        R: Into<Route> + Send
    {
        self.entity().async_create_secure_channel(route, trust_policy).await
    }
}

impl Issuer for Profile {
    fn get_signing_key(&mut self) -> Result<SecretKey> {
        self.entity().get_signing_key()
    }

    fn get_signing_public_key(&mut self) -> Result<SigningPublicKey> {
        self.entity().get_signing_public_key()
    }

    fn create_offer(&self, schema: &CredentialSchema) -> Result<CredentialOffer> {
        self.entity().create_offer(schema)
    }

    fn create_proof_of_possession(&self) -> Result<CredentialProof> {
        self.entity().create_proof_of_possession()
    }

    fn sign_credential<A: AsRef<[CredentialAttribute]>>(
        &self,
        schema: &CredentialSchema,
        attributes: A,
    ) -> Result<BbsCredential> {
        self.entity().sign_credential(schema, attributes)
    }

    fn sign_credential_request<A: AsRef<[(String, CredentialAttribute)]>>(
        &self,
        request: &CredentialRequest,
        schema: &CredentialSchema,
        attributes: A,
        offer_id: OfferId,
    ) -> Result<CredentialFragment2> {
        self.entity()
            .sign_credential_request(request, schema, attributes, offer_id)
    }
}

impl Holder for Profile {
    fn accept_credential_offer(
        &self,
        offer: &CredentialOffer,
        issuer_public_key: SigningPublicKey,
    ) -> Result<CredentialRequestFragment> {
        self.entity()
            .accept_credential_offer(offer, issuer_public_key)
    }

    fn combine_credential_fragments(
        &self,
        credential_fragment1: CredentialFragment1,
        credential_fragment2: CredentialFragment2,
    ) -> Result<BbsCredential> {
        self.entity()
            .combine_credential_fragments(credential_fragment1, credential_fragment2)
    }

    fn is_valid_credential(
        &self,
        credential: &BbsCredential,
        verifier_key: SigningPublicKey,
    ) -> Result<bool> {
        self.entity().is_valid_credential(credential, verifier_key)
    }

    fn create_credential_presentation(
        &self,
        credential: &BbsCredential,
        presentation_manifests: &PresentationManifest,
        proof_request_id: ProofRequestId,
    ) -> Result<CredentialPresentation> {
        self.entity().create_credential_presentation(
            credential,
            presentation_manifests,
            proof_request_id,
        )
    }

    fn add_credential(&mut self, credential: EntityCredential) -> Result<()> {
        self.entity().add_credential(credential)
    }

    fn get_credential(&mut self, credential: &Credential) -> Result<EntityCredential> {
        self.entity().get_credential(credential)
    }
}

impl Verifier for Profile {
    fn create_proof_request_id(&self) -> Result<ProofRequestId> {
        self.entity().create_proof_request_id()
    }

    fn verify_proof_of_possession(
        &self,
        signing_public_key: CredentialPublicKey,
        proof: CredentialProof,
    ) -> Result<bool> {
        self.entity()
            .verify_proof_of_possession(signing_public_key, proof)
    }

    fn verify_credential_presentation(
        &self,
        presentation: &CredentialPresentation,
        presentation_manifest: &PresentationManifest,
        proof_request_id: ProofRequestId,
    ) -> Result<bool> {
        self.entity().verify_credential_presentation(
            presentation,
            presentation_manifest,
            proof_request_id,
        )
    }
}
