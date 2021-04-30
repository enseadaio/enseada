# API Design Guidelines

This document describes the patterns and best practices used by Enseada when designing the public API.

The public API is an HTTP JSON API that allows an external system to interact and manipulate the state of an Enseada server.
The public API is served by a single process (called the API server or simply 'the server') and adheres to the principle of
[REST]. The API server is mostly stateless and relies on a backing persistent store (currently [CouchDB]) to store the actual
state of the system. This allows multiple concurrent instances of the API server to run in parallel, in order to load balance
incoming traffic and increase throughput and availability.

The public API is eventually consistent. This means that while changes to a resource definition are immediately persisted,
they will actually be applied some time in the (hopefully near) future. This is described in more details in the [Controllers](#controllers)
section of this document.

## Acknowledgments

A huge part of the public API is heavily inspired by the [Kubernetes API], especially around the schema versioning and definitions (like
the adoption of the GroupVersionKind mechanism to allow resource objects to be self-defined), and the spec/status dichotomy which
has proven an excellent tool to handle eventually consistent APIs.

## Resources

Resources are the fundamental building blocks of the public API. A resource represents an object whose state can be manipulated
through the API. All interactions with Enseada ultimately boils down to creating, updating and deleting resources.

Each resource is defined in terms of its [Type Metadata](#group-version-kind-typemeta) (or TypeMeta) which holds information such as the API Group and Version
the resource belongs to, as well as its Kind which identifies the schema of the resource it represents.  
Each resource also has an associated [Metadata](#metadata) section that holds common information such as the resource unique name,
creation and deletion timestamps and other meta information.

The rest of the resource object is highly dependent on the resource itself, but the vast majority of the public API adopts
a [Spec](#spec) and [Status](#status) structure, whereas the Spec defines the *desired state* of the resource, while the Status
represents its *actual current state*. This separation allows the API server to be basically resource agnostic, only implementing 
basic CRUD operations for all resources, while the actual business logic is implemented via reconciliation loops called [Controllers](#controllers)
that handle driving the state closed to the spec, updating the status accordingly.

This means the public API is inherently asynchronous, returning `202 Accepted` for most write operations. In the immediate moment
after a write, for example after updating a resource spec, the spec and status will be different because the controller
is yet to reconcile them. 

### Group Version Kind (TypeMeta)

An HTTP API needs to be versioned to allow for changes to be rolled out without impacting existing clients.
The Enseada public API groups related resources together, and versions are applied to the entire group. For example, 
the `core` group may have multiple versions, such as `v1alpha1`, `v1beta2` or `v2`. The version number is NOT semver compliant,
but instead uses an integer/modifier/integer system. The first character after the `v` marker is a non-negative integer which
MUST be incremented to represent backward-incompatible changes to an API resource. The rest of the string MAY be an alphabetic
sequence that represent a version qualifier, followed by a qualifier-specific non-negative integer. Common qualifiers are `alpha`, to denote
an unstable API, and `beta`, to denote a stable API that is still under evaluation and testing. A version without a qualifier
is considered stable/GA.

The Kind represents the schema of the resource. It has two forms: a singular, capitalized form used in the resource object
itself, and a plural, lowercase form used as a path segment in URLs.  
For example, the user kind is represented as `User` in JSON form, and `users` in plural form.

The Kind and the GroupVersion tuple are combined to form the API URL for a given resource, granting predictability and uniformity
to the API.

`/apis/core/v1alpha1/users`
```json
{
    "apiVersion": "core/v1alpha1",
    "kind": "User"
}
```

### Metadata

#### Name

### Spec and Status

## HTTP Endpoints

## Controllers

### Reconciliation process

[REST]: https://en.wikipedia.org/wiki/Representational_state_transfer
[CouchDB]: https://couchdb.apache.org
[Kubernetes API]: https://kubernetes.io/docs/concepts/overview/kubernetes-api/
